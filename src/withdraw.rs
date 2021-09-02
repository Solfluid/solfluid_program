use crate::payment_stream::PaymentStreams;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::transfer,
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
    let writing_account = next_account_info(accounts_iter)?;
    let reciver_account = next_account_info(accounts_iter)?;

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

    **writing_account.try_borrow_mut_lamports()? -= input_data.amount as u64;
    **reciver_account.try_borrow_mut_lamports()? += input_data.amount as u64;

    data_present.lamports_withdrawn += input_data.amount;

    data_present.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;
    Ok(())
}
