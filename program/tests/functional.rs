use sns_reputation::{
    entrypoint::process_instruction,
    instruction::vote,
    state::{reputation_score::ReputationScore, user_vote::UserVote, Tag},
};
use vote::Params;

use {
    solana_program::pubkey::Pubkey,
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{signer::Signer, transaction::Transaction},
};

pub mod common;

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

    let instruction = vote(
        vote::Accounts {
            system_program: &Pubkey::default(),
            voter: &payer_pubkey,
            reputation_state_account: &reputation_state,
            user_vote_state_account: &user_vote_key,
        },
        Params {
            user_key: votee,
            is_upvote: true,
        },
    );

    let recent_blockhash = prg_test_ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer_pubkey),
        &[&prg_test_ctx.payer],
        recent_blockhash,
    );

    prg_test_ctx
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let reputation_account = prg_test_ctx
        .banks_client
        .get_account(reputation_state)
        .await
        .unwrap()
        .unwrap();
    let parsed =
        ReputationScore::from_buffer(&reputation_account.data, Tag::ReputationScore).unwrap();

    let user_vote_account = prg_test_ctx
        .banks_client
        .get_account(user_vote_key)
        .await
        .unwrap()
        .unwrap();

    let parsed_user_vote = UserVote::from_buffer(&user_vote_account.data, Tag::UserVote).unwrap();

    assert_eq!(
        parsed,
        ReputationScore {
            nonce: reputation_state_nonce,
            upvote: 1,
            downvote: 0
        }
    );

    // Check that UserVote contains all expected metadata
    assert_eq!(
        parsed_user_vote,
        UserVote {
            value: true,
            voter: payer_pubkey,
            votee
        }
    );

    // ============================================
    // Now try to vote same value and check that user's vote will actually
    // be cancelled and rent value will return back to payer

    // Will be used below
    let balance_after_vote = prg_test_ctx
        .banks_client
        .get_balance(payer_pubkey)
        .await
        .unwrap();

    let instruction = vote(
        vote::Accounts {
            system_program: &Pubkey::default(),
            voter: &payer_pubkey,
            reputation_state_account: &reputation_state,
            user_vote_state_account: &user_vote_key,
        },
        Params {
            user_key: votee,
            is_upvote: true,
        },
    );

    prg_test_ctx.warp_to_slot(2).unwrap();

    let recent_blockhash = prg_test_ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer_pubkey),
        &[&prg_test_ctx.payer],
        recent_blockhash,
    );

    prg_test_ctx
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let reputation_account = prg_test_ctx
        .banks_client
        .get_account(reputation_state)
        .await
        .unwrap()
        .unwrap();

    let parsed =
        ReputationScore::from_buffer(&reputation_account.data, Tag::ReputationScore).unwrap();

    assert_eq!(
        parsed,
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
        balance_after_undo_vote > balance_after_vote,
        "❌ Rent was not returned to the user!"
    );
}
