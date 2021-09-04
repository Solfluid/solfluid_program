mod close_stream;
mod create_stream;
mod payment_stream;
mod unstake_token;
mod withdraw;
use close_stream::close_stream;
use create_stream::create_stream;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use unstake_token::unstake_tokens;
use withdraw::withdraw;

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data[0] == 1 {
        return create_stream(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }
    if instruction_data[0] == 2 {
        return unstake_tokens(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }
    if instruction_data[0] == 3 {
        return withdraw(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }
    if instruction_data[0] == 4 {
        return close_stream(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }
    Err(ProgramError::InvalidInstructionData)
}
entrypoint!(process_instruction);

// Sanity tests
#[cfg(test)]
mod test {

    #[test]
    fn test_sanity() {}
}
