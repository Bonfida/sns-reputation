use {
    num_derive::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum SnsReputationError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("Data type mismatch")]
    DataTypeMismatch,
    #[error("Wrong account owner")]
    WrongOwner,
    #[error("Account is uninitialized")]
    Uninitialized,
    #[error("Already voted")]
    AlreadyVoted,
    #[error("No vote exists")]
    NoVoteExists,
}

impl From<SnsReputationError> for ProgramError {
    fn from(e: SnsReputationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SnsReputationError {
    fn type_of() -> &'static str {
        "SnsReputationError"
    }
}
