use crate::payment_stream::PaymentStreams;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    stake,
    sysvar::Sysvar,
};

pub fn create_stream(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    // write true
    let writing_account = next_account_info(accounts_iter)?;
    let stake_account = next_account_info(accounts_iter)?;
    // is signer true
    let senders_account = next_account_info(accounts_iter)?;
    let reciver_account = next_account_info(accounts_iter)?;
    let vote_account = next_account_info(accounts_iter)?;
    let clock_account = next_account_info(accounts_iter)?;
    let stake_history = next_account_info(accounts_iter)?;
    let config_account = next_account_info(accounts_iter)?;

    if !senders_account.is_signer {
        msg!("Sender account should be signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    if writing_account.owner != program_id {
        msg!("Writter account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    if senders_account.key == reciver_account.key {
        msg!("Seriouly if you are this dumb, I am going to lose my shit");
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut input_data = PaymentStreams::try_from_slice(&instruction_data)
        .expect("instruction data serialization didn't worked");

    let time: i64 = Clock::get()?.unix_timestamp;
    if input_data.start_time < time {
        msg!("Start time should not be less then current time");
        return Err(ProgramError::InvalidInstructionData);
    }

    if input_data.amount_second < 0 {
        msg!("Can't procced");
        return Err(ProgramError::InvalidInstructionData);
    }

    if input_data.start_time >= input_data.end_time {
        msg!("Start time should'nt be greater than end time");
        return Err(ProgramError::InvalidInstructionData);
    }

    if input_data.from != senders_account.key.clone()
        && input_data.to != reciver_account.key.clone()
    {
        msg!("Incorrect input instruction");
        return Err(ProgramError::InvalidInstructionData);
    }
    //170 length of data
    let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());
    let total_amount_to_be_streamed =
        ((input_data.end_time - input_data.start_time) * input_data.amount_second) as u64;
    if **writing_account.lamports.borrow_mut() + **stake_account.lamports.borrow_mut()
        < total_amount_to_be_streamed + rent_exemption
    {
        msg!("Can't procced");
        return Err(ProgramError::InvalidAccountData);
    }
    let delegate_instruction = stake::instruction::delegate_stake(
        stake_account.key,
        writing_account.key,
        &input_data.vote_right_to,
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
        &[&[input_data.seed.as_bytes()]],
    )
    .expect("delegation failed");

    input_data.is_delegated = true;
    input_data.stake_account = *stake_account.key;
    input_data.lamports_withdrawn = 0;
    input_data.is_active = true;

    input_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;
    Ok(())
}
