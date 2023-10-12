use crate::{error::SnsReputationError, processor::Processor};

use {
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo, decode_error::DecodeError, entrypoint::ProgramResult, msg,
        program_error::PrintProgramError, pubkey::Pubkey,
    },
};

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

/// The entrypoint to the program
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint");
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<SnsReputationError>();
        return Err(error);
    }
    Ok(())
}

impl PrintProgramError for SnsReputationError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            SnsReputationError::AlreadyInitialized => {
                msg!("Error: This account is already initialized")
            }
            SnsReputationError::DataTypeMismatch => msg!("Error: Data type mismatch"),
            SnsReputationError::WrongOwner => msg!("Error: Wrong account owner"),
            SnsReputationError::Uninitialized => msg!("Error: Account is uninitialized"),
            SnsReputationError::AlreadyVoted => msg!("Error: Already voted"),
            SnsReputationError::NoVoteExists => msg!("Error: No vote exists"),
            SnsReputationError::MissingStakeAccount => {
                msg!("Error: A valid stake account is necessary to be allowed to vote")
            }
            SnsReputationError::InvalidStakeAccount => {
                msg!("Error: A valid stake account is necessary to be allowed to vote")
            }
            SnsReputationError::CannotVoteForYourself => {
                msg!("Error: You cannot vote for yourself")
            }
        }
    }
}
