use bonfida_utils::BorshSize;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::SnsReputationError;

use super::{Tag, VoteValue};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, BorshSize, Default)]
#[allow(missing_docs)]
#[repr(C)]
pub struct UserVote {
    /// User's vote over votee
    pub value: VoteValue,
    /// Votee address, stored as metadata
    pub votee: Pubkey,
    /// Voter address, stored as metadata
    pub voter: Pubkey,
}

#[allow(missing_docs)]
impl UserVote {
    pub fn from_buffer(buffer: &[u8], expected_tag: super::Tag) -> Result<Self, ProgramError> {
        let (tag, mut buffer) = buffer.split_at(8);
        if *bytemuck::from_bytes::<u64>(tag) != expected_tag as u64 {
            return Err(SnsReputationError::DataTypeMismatch.into());
        }
        Ok(Self::deserialize(&mut buffer)?)
    }

    pub fn find_key(program_id: &Pubkey, addresses: &(Pubkey, Pubkey)) -> (Pubkey, u8) {
        let (user_address, voter) = addresses;

        let seeds: &[&[u8]] = &[user_address.as_ref(), voter.as_ref()];
        Pubkey::find_program_address(seeds, program_id)
    }

    pub fn save(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        // Skip first 8 bytes and store other data after them
        self.serialize(&mut (&mut dst[8..]))?;
        // First Tag data in the first 8 bytes (u64 size) to represent what kind of data stored in next bytes
        (Tag::UserVote as u64).serialize(&mut (&mut dst[..]))?;
        Ok(())
    }
}
