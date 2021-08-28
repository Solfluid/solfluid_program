use crate::payment_stream::PaymentStreams;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
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
    if input_data.from != senders_account.key.clone()
        && input_data.to != reciver_account.key.clone()
    {
        msg!("Incorrect input instruction");
        return Err(ProgramError::InvalidInstructionData);
    }
    input_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;
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
        let mut lamports1 = 0;
        let mut data1 = [];
        let mut lamports2 = 0;
        let mut data2 = [];
        let owner1 = Pubkey::new_unique();
        let owner2 = Pubkey::new_unique();
        let sender_account = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports1,
            &mut data1,
            &owner1,
            false,
            Epoch::default(),
        );
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

        let accounts = vec![writer_account, sender_account, reciver_account];
        let data_to_send = PaymentStreams {
            from: accounts[1].key.clone(),
            to: accounts[2].key.clone(),
            end_time: 1121212,
            start_time: 212121,
            lamports_withdrawn: 0,
            amount_second: 12121,
        };

        let v = data_to_send.try_to_vec().unwrap();

        assert_eq!(create_stream(&program_id, &accounts, &v), Ok(()));

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
