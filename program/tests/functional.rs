use sns_reputation::{
    entrypoint::process_instruction,
    error::SnsReputationError,
    instruction::vote,
    state::{reputation_score::ReputationScore, user_vote::UserVote, Tag, VoteValue},
};
use vote::Params;

use {
    solana_program::{instruction::InstructionError, pubkey::Pubkey},
    solana_program_test::{processor, BanksClientError, ProgramTest, ProgramTestContext},
    solana_sdk::{
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
};

pub mod common;

async fn process_vote(
    prg_test_ctx: &mut ProgramTestContext,
    payer: Pubkey,
    reputation_state_account: Pubkey,
    user_vote_state_account: Pubkey,
    vote_value: VoteValue,
    votee: Pubkey,
    current_slot: &mut u8,
) -> Result<(), BanksClientError> {
    // Move solana to a few slots in the future so recent_blockhash will differ
    // for each transaction
    *current_slot += 2;
    prg_test_ctx.warp_to_slot(*current_slot as u64).unwrap();

    let instruction = vote(
        vote::Accounts {
            system_program: &Pubkey::default(),
            voter: &payer,
            reputation_state_account: &reputation_state_account,
            user_vote_state_account: &user_vote_state_account,
            voter_stake_accounts: &[],
        },
        Params {
            user_key: votee,
            vote_value,
        },
    );

    let recent_blockhash = prg_test_ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer),
        &[&prg_test_ctx.payer],
        recent_blockhash,
    );

    prg_test_ctx
        .banks_client
        .process_transaction(transaction)
        .await
}

async fn fetch_reputation_score(
    prg_test_ctx: &mut ProgramTestContext,
    reputation_state: Pubkey,
) -> ReputationScore {
    let reputation_account = prg_test_ctx
        .banks_client
        .get_account(reputation_state)
        .await
        .unwrap()
        .unwrap();

    ReputationScore::from_buffer(&reputation_account.data, Tag::ReputationScore).unwrap()
}

async fn fetch_user_vote(prg_test_ctx: &mut ProgramTestContext, user_vote_key: Pubkey) -> UserVote {
    let user_vote_account = prg_test_ctx
        .banks_client
        .get_account(user_vote_key)
        .await
        .unwrap()
        .unwrap();

    UserVote::from_buffer(&user_vote_account.data, Tag::UserVote).unwrap()
}

#[tokio::test]
async fn test_voting() {
    let program_test = ProgramTest::new(
        "sns_reputation",
        sns_reputation::ID,
        processor!(process_instruction),
    );
    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    let votee = Pubkey::new_unique();
    let payer_pubkey = prg_test_ctx.payer.pubkey();
    let (reputation_state, reputation_state_nonce) =
        ReputationScore::find_key(&sns_reputation::ID, &votee);
    let (user_vote_key, _) = UserVote::find_key(&sns_reputation::ID, &(votee, payer_pubkey));
    let mut current_slot: u8 = 1;

    // ============================================
    // Check that user can Upvote

    process_vote(
        &mut prg_test_ctx,
        payer_pubkey,
        reputation_state,
        user_vote_key,
        VoteValue::Upvote,
        votee,
        &mut current_slot,
    )
    .await
    .unwrap();

    let parsed_reputation_score = fetch_reputation_score(&mut prg_test_ctx, reputation_state).await;
    let parsed_user_vote = fetch_user_vote(&mut prg_test_ctx, user_vote_key).await;

    assert_eq!(
        parsed_reputation_score,
        ReputationScore {
            nonce: reputation_state_nonce,
            upvote: 1,
            downvote: 0,
        }
    );

    // Check that UserVote contains all expected metadata
    assert_eq!(
        parsed_user_vote,
        UserVote {
            value: VoteValue::Upvote as i64,
            voter: payer_pubkey,
            votee,
        }
    );

    // ============================================
    // Now try to vote same value and check that program returned an error

    let tx_result = process_vote(
        &mut prg_test_ctx,
        payer_pubkey,
        reputation_state,
        user_vote_key,
        VoteValue::Upvote,
        votee,
        &mut current_slot,
    )
    .await;

    if let Err(BanksClientError::TransactionError(TransactionError::InstructionError(
        0,
        InstructionError::Custom(n),
    ))) = tx_result
    {
        assert_eq!(
            n,
            SnsReputationError::AlreadyVoted as u32,
            "❌ Error is not correct!"
        )
    } else {
        panic!();
    };

    // ============================================
    // Now try to change vote to opposite one

    let opposite_vote = VoteValue::Downvote;

    process_vote(
        &mut prg_test_ctx,
        payer_pubkey,
        reputation_state,
        user_vote_key,
        // Change to Downvote
        opposite_vote,
        votee,
        &mut current_slot,
    )
    .await
    .unwrap();

    let parsed_reputation_score = fetch_reputation_score(&mut prg_test_ctx, reputation_state).await;
    let parsed_user_vote = fetch_user_vote(&mut prg_test_ctx, user_vote_key).await;

    assert_eq!(
        parsed_reputation_score,
        ReputationScore {
            nonce: reputation_state_nonce,
            upvote: 0,
            downvote: 1
        }
    );

    // Check that UserVote contains all expected metadata
    assert_eq!(
        parsed_user_vote,
        UserVote {
            value: opposite_vote as i64,
            voter: payer_pubkey,
            votee
        },
        "❌ New opposite vote is incorrect!"
    );

    // ============================================
    // Now try to undo vote and check that rent is returned back

    let balance_before_undo_vote = prg_test_ctx
        .banks_client
        .get_balance(payer_pubkey)
        .await
        .unwrap();

    process_vote(
        &mut prg_test_ctx,
        payer_pubkey,
        reputation_state,
        user_vote_key,
        // Change to Downvote
        VoteValue::NoVote,
        votee,
        &mut current_slot,
    )
    .await
    .unwrap();

    let parsed_reputation_score = fetch_reputation_score(&mut prg_test_ctx, reputation_state).await;

    assert_eq!(
        parsed_reputation_score,
        ReputationScore {
            nonce: reputation_state_nonce,
            upvote: 0,
            downvote: 0
        }
    );

    let user_vote_account = prg_test_ctx
        .banks_client
        .get_account(user_vote_key)
        .await
        .unwrap();

    assert!(
        user_vote_account.is_none(),
        "❌ user_vote_account still exists!"
    );

    // Check that after undo balance was increased, so some rent is returned
    // back to the user
    let balance_after_undo_vote = prg_test_ctx
        .banks_client
        .get_balance(payer_pubkey)
        .await
        .unwrap();

    assert!(
        balance_after_undo_vote > balance_before_undo_vote,
        "❌ Rent was not returned to the user!"
    );

    // ============================================
    // Test that user cannot vote with undo when there's nothing to undo

    let tx_result = process_vote(
        &mut prg_test_ctx,
        payer_pubkey,
        reputation_state,
        user_vote_key,
        VoteValue::NoVote,
        votee,
        &mut current_slot,
    )
    .await;

    if let Err(BanksClientError::TransactionError(TransactionError::InstructionError(
        0,
        InstructionError::Custom(n),
    ))) = tx_result
    {
        assert_eq!(
            n,
            SnsReputationError::NoVoteExists as u32,
            "❌ Error is not correct!"
        )
    } else {
        panic!();
    };
}
