use crate::payment_stream::PaymentStreams;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn create_stream(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let writing_account = next_account_info(accounts_iter)?;
    let senders_account = next_account_info(accounts_iter)?;
    let reciver_account = next_account_info(accounts_iter)?;

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
    let input_data: PaymentStreams = match BorshDeserialize::try_from_slice(instruction_data) {
        Ok(x) => x,
        Err(e) => {
            msg!("Invalid Input {}", e.to_string());
            return Err(ProgramError::InvalidInstructionData);
        }
    };
    let time: i64 = Clock::get()?.unix_timestamp;
    if input_data.start_time < time {
        msg!("Start time should not be less then current time");
        return Err(ProgramError::InvalidInstructionData);
    }

    if input_data.start_time >= input_data.end_time {
        msg!("Incorrect input instruction");
        return Err(ProgramError::InvalidInstructionData);
    }

    if !input_data.is_active {
        msg!("Incorrect input instruction");
        return Err(ProgramError::InvalidInstructionData);
    }
    if input_data.from != senders_account.key.clone()
        && input_data.to != reciver_account.key.clone()
    {
        msg!("Incorrect input instruction");
        return Err(ProgramError::InvalidInstructionData);
    }
    let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());
    let total_amount_to_be_streamed =
        ((input_data.end_time - input_data.start_time) * input_data.amount_second) as u64;
    if **writing_account.lamports.borrow_mut() < total_amount_to_be_streamed + rent_exemption {
        msg!("Can't procced");
        return Err(ProgramError::InvalidAccountData);
    }

    input_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;

    Ok(())
}
