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
    system_instruction::transfer,
    sysvar::Sysvar,
};
use std::convert::TryInto;
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ReciverRewardPercentage {
    pub percentage: u8,
}

pub fn close_stream(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let administrator = next_account_info(accounts_iter)?;
    let writing_account = next_account_info(accounts_iter)?;
    let sender_account = next_account_info(accounts_iter)?;
    let reciver_account = next_account_info(accounts_iter)?;

    if !sender_account.is_signer {
        msg!("Sender account should be signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    if writing_account.owner != program_id && administrator.owner != program_id {
        msg!("Writter account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data_present = PaymentStreams::try_from_slice(&writing_account.data.borrow())
        .expect("account data serialization didn't worked");

    if data_present.to != *reciver_account.key {
        msg!(
            "You can't get money from this stream {:?} , {:?}",
            data_present.to,
            *reciver_account.key
        );
        return Err(ProgramError::InvalidAccountData);
    }

    if data_present.from != *sender_account.key {
        msg!("You can't get money from this stream");
        return Err(ProgramError::InvalidAccountData);
    }
    if !data_present.is_active {
        msg!("not active already");
        return Err(ProgramError::InvalidInstructionData);
    }

    let input_data = ReciverRewardPercentage::try_from_slice(instruction_data)
        .expect("Instruction Data didn't worked");

    if input_data.percentage > 100 {
        msg!("invalid input");
        return Err(ProgramError::InvalidInstructionData);
    }
    let time: i64 = Clock::get()?.unix_timestamp;
    let mut lamport_streamed_to_reciver: i64 = 0;
    if time > data_present.start_time {
        lamport_streamed_to_reciver = data_present.amount_second
            * (std::cmp::min(time, data_present.end_time) - data_present.start_time)
            - data_present.lamports_withdrawn;
    }

    let rent_taken: i64 = Rent::get()?.minimum_balance(writing_account.data_len()) as i64;

    let writing_account_balance: i64 = (**writing_account.lamports.borrow_mut())
        .try_into()
        .unwrap();

    let totalamount_streamed: i64 =
        data_present.amount_second * (data_present.end_time - data_present.start_time);

    let yield_earned = writing_account_balance - totalamount_streamed - rent_taken;

    let reward_perctage_reciver: i64 = (input_data.percentage as f64 / 100f64) as i64;
    let reward_earned_reciver: i64 = yield_earned * reward_perctage_reciver;

    let reward_earned_sender: i64 = yield_earned - reward_perctage_reciver;

    transfer(
        writing_account.key,
        reciver_account.key,
        (lamport_streamed_to_reciver + reward_earned_reciver) as u64,
    );

    transfer(
        writing_account.key,
        sender_account.key,
        (rent_taken + totalamount_streamed - lamport_streamed_to_reciver + reward_earned_sender)
            as u64,
    );

    data_present.lamports_withdrawn += lamport_streamed_to_reciver;
    data_present.is_active = false;

    data_present.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;

    Ok(())
}
