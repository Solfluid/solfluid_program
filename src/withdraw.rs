use crate::payment_stream::PaymentStreams;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    stake,
    sysvar::Sysvar,
};

#[derive(BorshSerialize, BorshSchema, BorshDeserialize, Debug, Clone)]
pub struct WithdrawAmount {
    amount: i64,
}

pub fn withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    //write true
    let writing_account = next_account_info(accounts_iter)?;
    //write true
    let stake_account = next_account_info(accounts_iter)?;
    let vote_account = next_account_info(accounts_iter)?;
    let clock_account = next_account_info(accounts_iter)?;
    let stake_history = next_account_info(accounts_iter)?;
    // is signer true
    let reciver_account = next_account_info(accounts_iter)?;
    let config_account = next_account_info(accounts_iter)?;

    if !reciver_account.is_signer {
        msg!("Sender account should be signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    if writing_account.owner != program_id {
        msg!("Writter account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut data_present = PaymentStreams::try_from_slice(&writing_account.data.borrow())
        .expect("account data serialization didn't worked");

    if !data_present.is_active {
        msg!("Invalid input data");
        return Err(ProgramError::InvalidInstructionData);
    }

    if data_present.to != *reciver_account.key {
        msg!("You can't get money from this stream");
        return Err(ProgramError::InvalidAccountData);
    }

    let input_data =
        WithdrawAmount::try_from_slice(instruction_data).expect("Instruction Data didn't worked");
    let time: i64 = Clock::get()?.unix_timestamp;

    let total_amount_received = data_present.amount_second
        * (std::cmp::min(time, data_present.end_time) - data_present.start_time)
        - data_present.lamports_withdrawn;

    if input_data.amount > total_amount_received
        || input_data.amount < 0
        || total_amount_received < 0
    {
        msg!("Insufficent balance");
        return Err(ProgramError::InsufficientFunds);
    }
    if data_present.is_delegated {
        msg!("stakes are delegated can't withdraw");
        return Err(ProgramError::InsufficientFunds);
    }

    if time < data_present.delegate_time {
        msg!("undelegated stakes will need some time to be added to account");
        return Err(ProgramError::InvalidArgument);
    }

    let withdraw_instruction = stake::instruction::withdraw(
        stake_account.key,
        writing_account.key,
        writing_account.key,
        input_data.amount as u64,
        None,
    );

    invoke_signed(
        &withdraw_instruction,
        &[
            stake_account.to_owned(),
            clock_account.to_owned(),
            writing_account.to_owned(),
        ],
        &[&[data_present.seed.as_bytes()]],
    )
    .expect("Withdraw failed");

    **writing_account.try_borrow_mut_lamports()? -= input_data.amount as u64;
    **reciver_account.try_borrow_mut_lamports()? += input_data.amount as u64;
    data_present.lamports_withdrawn += input_data.amount;

    let delegate_instruction = stake::instruction::delegate_stake(
        stake_account.key,
        writing_account.key,
        &data_present.vote_right_to,
    );

    invoke_signed(
        &delegate_instruction,
        &[
            stake_account.to_owned(),
            vote_account.to_owned(),
            clock_account.to_owned(),
            stake_history.to_owned(),
            config_account.to_owned(),
            writing_account.to_owned(),
        ],
        &[&[data_present.seed.as_bytes()]],
    )
    .expect("delegation failed");

    data_present.is_delegated = true;

    data_present.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;
    Ok(())
}
