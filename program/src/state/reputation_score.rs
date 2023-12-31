use bonfida_utils::BorshSize;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::SnsReputationError;

use super::Tag;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, BorshSize, Default)]
#[allow(missing_docs)]
#[repr(C)]
pub struct ReputationScore {
    /// Nonce
    pub nonce: u8,
    /// Upvotes amount
    pub upvote: u64,
    /// Downvotes amount
    pub downvote: u64,
}

#[allow(missing_docs)]
impl ReputationScore {
    pub fn from_buffer(buffer: &[u8], expected_tag: super::Tag) -> Result<Self, ProgramError> {
        let (tag, mut buffer) = buffer.split_at(8);
        if *bytemuck::from_bytes::<u64>(tag) != expected_tag as u64 {
            return Err(SnsReputationError::DataTypeMismatch.into());
        }
        Ok(Self::deserialize(&mut buffer)?)
    }

    pub fn find_key(program_id: &Pubkey, user_address: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[user_address.as_ref()];
        Pubkey::find_program_address(seeds, program_id)
    }

    // Stores data
    pub fn save(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        // Skip first 8 bytes and store other data after them
        self.serialize(&mut (&mut dst[8..]))?;
        // First Tag data in the first 8 bytes (u64 size) to represent what kind of data stored in next bytes
        (Tag::ReputationScore as u64).serialize(&mut (&mut dst[..]))?;
        Ok(())
    }
}
