use crate::error::SnsReputationError;

use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

pub mod reputation_score;

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq)]
#[repr(u64)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    ReputationScore,
}
