use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
};

pub mod reputation_score;
pub mod user_vote;

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq)]
#[repr(u64)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    ReputationScore,
    UserVote,
}

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum VoteValue {
    NoVote,
    Downvote,
    Upvote,
}

impl Default for VoteValue {
    fn default() -> Self {
        VoteValue::NoVote
    }
}
