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

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq, Debug, Clone, Copy, Default)]
// Borsh only works with u8 in enums
#[repr(i64)]
#[allow(missing_docs)]
pub enum VoteValue {
    #[default]
    NoVote = 0,
    Downvote = -1,
    Upvote = 1,
}

#[cfg(test)]
#[tokio::test]
pub async fn test_stake_parsing() {
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_program::{pubkey, stake::state::StakeState};
    let solana_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_owned());
    let stake_account = pubkey!("GZtaL9bwzAWWyiNWxtNPkgRGpmF8LLeLVA3FbhLv14Pu");
    let stake_authority = pubkey!("J6QDztZCegYTWnGUYtjqVS9d7AZoS43UbEQmMcdGeP5s");
    let account = solana_client.get_account(&stake_account).await.unwrap();
    assert_eq!(account.owner, solana_program::stake::program::ID);
    let stake =
        solana_program::stake::state::StakeState::deserialize(&mut (&account.data as &[u8]))
            .unwrap();
    if let StakeState::Stake(m, s) = stake {
        assert_eq!(m.authorized.staker, stake_authority);
    } else {
        panic!()
    }
}
