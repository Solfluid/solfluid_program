use std::io::ErrorKind;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PaymentStreams {
    pub end_time: u128,
    pub start_time: u128,
    pub amount_second: u128,
}

impl PaymentStreams {
    fn new() -> PaymentStreams {
        PaymentStreams {
            amount_second: 0,
            start_time: 0,
            end_time: 0,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AllData {
    pub to: Pubkey,
    pub from: Pubkey,
    pub streams: Vec<PaymentStreams>,
}

entrypoint!(create_stream);

fn get_initial_data(from: &Pubkey, to: &Pubkey) -> AllData {
    AllData {
        from: from.clone(),
        to: to.clone(),
        streams: [PaymentStreams::new(), PaymentStreams::new()].to_vec(),
    }
}

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
    let input_data: PaymentStreams = match BorshDeserialize::try_from_slice(instruction_data) {
        Ok(x) => x,
        Err(e) => {
            msg!("Invalid Input {}", e.to_string());
            return Err(ProgramError::InvalidInstructionData);
        }
    };
    let data_already: AllData =
        match BorshDeserialize::try_from_slice(&writing_account.data.borrow_mut()) {
            Ok(x) => x,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    get_initial_data(&senders_account.key, &reciver_account.key)
                } else {
                    panic!("Unknown error decoding account data {:?}", e)
                }
            }
        };
    let mut found_one = false;
    for ps in data_already.streams {
        if ps.end_time < input_data.start_time {
            found_one = true;
            continue;
        }
    }
    if !found_one {
        let error =
            "Cannot create More then 2 active streams between same reciver and sender account"
                .to_string();
        return Err(ProgramError::BorshIoError(error));
    }
    // accounts

    msg!("decoded {:?}", input_data);
    Ok(())
}

// Sanity tests
#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;
    use std::{mem, ptr::null};

    #[test]
    fn test_sanity() {
        let program_id = Pubkey::new_unique();
        let key = Pubkey::default();
        let mut lamports1 = 0;
        let mut data1 = [];
        let mut lamports2 = 0;
        let mut data2 = [];
        let owner = Pubkey::default();
        let senderAccount = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports1,
            &mut data1,
            &owner,
            false,
            Epoch::default(),
        );
        let reciverAccount = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports2,
            &mut data2,
            &owner,
            false,
            Epoch::default(),
        );

        let accounts = vec![senderAccount, reciverAccount];
        let data_to_send = PaymentStreams {
            end_time: 1121212,
            start_time: 212121,
            amount_second: 12121,
        };
        let v = data_to_send.try_to_vec().unwrap();

        assert_eq!(
            create_stream(&program_id, &accounts, &v),
            Err(ProgramError::InvalidAccountData)
        );

        // assert_eq!(
        //     GreetingAccount::try_from_slice(&accounts[0].data.borrow())
        //         .unwrap()
        //         .counter,
        //     0
        // );
        // process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        // assert_eq!(
        //     GreetingAccount::try_from_slice(&accounts[0].data.borrow())
        //         .unwrap()
        //         .counter,
        //     1
        // );
        // process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        // assert_eq!(
        //     GreetingAccount::try_from_slice(&accounts[0].data.borrow())
        //         .unwrap()
        //         .counter,
        //     2
        // );
    }
}
