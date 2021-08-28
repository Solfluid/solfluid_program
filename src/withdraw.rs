use std::convert::TryInto;

use crate::payment_stream::PaymentStreams;
use borsh::{BorshDeserialize, BorshSerialize};
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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
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

    if writing_account.owner != program_id {
        msg!("Writter account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut data_present: PaymentStreams =
        match BorshDeserialize::try_from_slice(writing_account.data.take()) {
            Ok(x) => x,
            Err(er) => {
                msg!("{}", er);
                return Err(ProgramError::InvalidAccountData);
            }
        };
    if !data_present.is_active {
        msg!("Invalid input data");
        return Err(ProgramError::InvalidInstructionData);
    }

    if data_present.to != reciver_account.owner.clone() {
        msg!("You can't get money from this stream");
        return Err(ProgramError::InvalidAccountData);
    }

    let input_data: WithdrawAmount = match BorshDeserialize::try_from_slice(instruction_data) {
        Ok(a) => a,
        Err(a) => {
            msg!("Invalid input data, {}", a);
            return Err(ProgramError::InvalidInstructionData);
        }
    };
    if input_data.amount
        > data_present.amount_second
            * (Clock::get().unwrap().unix_timestamp - data_present.start_time)
            - data_present.lamports_withdrawn
        && input_data.amount < 0
    {
        msg!("Insufficent balance");
        return Err(ProgramError::InsufficientFunds);
    }

    transfer(
        writing_account.key,
        reciver_account.key,
        input_data.amount.try_into().unwrap(),
    );
    data_present.lamports_withdrawn += input_data.amount;

    data_present.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;
    Ok(())
}
