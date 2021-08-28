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
    msg!("{}", Clock::get().unwrap().unix_timestamp);
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

// Sanity tests
#[cfg(test)]
mod test {
    use super::*;
    use borsh::BorshSerialize;
    use solana_program::clock::Epoch;

    #[test]
    fn test_sanity() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let mut lamports2 = 0;
        let mut data2 = [];
        let owner2 = Pubkey::new_unique();
        let reciver_account = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports2,
            &mut data2,
            &owner2,
            false,
            Epoch::default(),
        );
        let mut lamports3 = 0;
        let mut data3 = [0u8; 112];
        let writer_account = AccountInfo::new(
            &program_id,
            false,
            false,
            &mut lamports3,
            &mut data3,
            &program_id,
            false,
            Epoch::default(),
        );

        let accounts = vec![writer_account, reciver_account];
        let data_to_send = PaymentStreams {
            from: accounts[1].owner.clone(),
            to: accounts[2].owner.clone(),
            end_time: 1121212,
            start_time: 212121,
            lamports_withdrawn: 0,
            amount_second: 12121,
        };

        let _v = data_to_send.try_to_vec().unwrap();

        // assert_eq!(create_stream(&program_id, &accounts, &v), Ok(()));

        let data_changed: PaymentStreams =
            match BorshDeserialize::try_from_slice(accounts[0].data.take()) {
                Ok(x) => x,
                Err(error) => {
                    msg!("{}", error);
                    panic!("error");
                }
            };

        msg!("{:?}", data_changed);

        // assert_eq!(
        //     GreetingAccount::try_from_slice(&accounts[0].data.borrow())
        //         .unwrap()
        //         .counter,
        //     0
        // );
        // process_instruction(&program_id, &accounts, &instruction_data).unwrap();
    }
}
