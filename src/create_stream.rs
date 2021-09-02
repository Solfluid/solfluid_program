use crate::payment_stream::{PaymentStreams, PaymentStreamsInput};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
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

    let input_data = PaymentStreamsInput::try_from_slice(instruction_data)
        .expect("instruction data serialization didn't worked");

    let clock = Clock::get()?;
    let time: i64 = clock.unix_timestamp;
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
    //size of struct is 104
    let rent_exemption = Rent::get()?.minimum_balance(instruction_data.len());
    let total_amount_to_be_streamed =
        ((input_data.end_time - input_data.start_time) * input_data.amount_second) as u64;
    if **writing_account.lamports.borrow_mut() < total_amount_to_be_streamed + rent_exemption {
        msg!("Can't procced");
        return Err(ProgramError::InvalidAccountData);
    }
    let stake_pubkey = Pubkey::create_with_seed(writing_account.key, "seed", &stake::config::id())?;

    let authorized = stake::state::Authorized {
        staker: *writing_account.key,
        withdrawer: *writing_account.key,
    };
    let lockup = stake::state::Lockup {
        custodian: *writing_account.key,
        epoch: clock.epoch_start_timestamp as u64,
        unix_timestamp: input_data.start_time,
    };

    stake::instruction::create_account_and_delegate_stake(
        &writing_account.key,
        &stake_pubkey,
        &input_data.stake_token_to, //
        &authorized,
        &lockup,
        total_amount_to_be_streamed,
    );
    let data_to_write = PaymentStreams {
        amount_second: input_data.amount_second,
        end_time: input_data.end_time,
        from: input_data.from,
        is_active: true,
        lamports_withdrawn: 0,
        stake_pubkey: stake_pubkey,
        to: input_data.to,
        start_time: input_data.start_time,
    };
    data_to_write.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;
    Ok(())
}
