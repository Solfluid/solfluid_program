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
pub struct UnstakeAmount {
    amount: i64,
}

pub fn unstake_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let writing_account = next_account_info(accounts_iter)?;
    let stake_account = next_account_info(accounts_iter)?;
    let clock_account = next_account_info(accounts_iter)?;
    let sender_account = next_account_info(accounts_iter)?;
    let reciver_account = next_account_info(accounts_iter)?;

    if !sender_account.is_signer && !reciver_account.is_signer {
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

    let input_data =
        UnstakeAmount::try_from_slice(instruction_data).expect("Instruction Data didn't worked");

    let time = Clock::get()?.unix_timestamp;

    let total_earned = (data_present.end_time - time) * data_present.amount_second
        - data_present.lamports_withdrawn;

    if input_data.amount > total_earned {
        return Err(ProgramError::InvalidArgument);
    }
    let unstake_tokens =
        stake::instruction::deactivate_stake(stake_account.key, writing_account.key);

    invoke_signed(
        &unstake_tokens,
        &[
            stake_account.to_owned(),
            clock_account.to_owned(),
            writing_account.to_owned(),
        ],
        &[&[data_present.seed.as_bytes()]],
    )
    .expect("invoke failed");

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
    data_present.delegate_time = Clock::get()?.epoch_start_timestamp;
    data_present.is_delegated = false;
    data_present.lamports_withdrawn += input_data.amount;
    data_present.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;

    Ok(())
}
