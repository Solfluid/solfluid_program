mod close_stream;
mod create_stream;
mod payment_stream;
mod withdraw;
use close_stream::close_stream;
use create_stream::create_stream;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use withdraw::withdraw;

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if accounts.len() == 3 {
        return create_stream(program_id, accounts, instruction_data);
    }
    if accounts.len() == 2 {
        return withdraw(program_id, accounts, instruction_data);
    }
    if accounts.len() == 4 {
        return close_stream(program_id, accounts, instruction_data);
    }

    Err(ProgramError::InvalidInstructionData)
}
entrypoint!(process_instruction);

// Sanity tests
#[cfg(test)]
mod test {
    use crate::payment_stream::PaymentStreams;

    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::{clock::Epoch, msg};

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
            is_active: true,
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
    }
}
