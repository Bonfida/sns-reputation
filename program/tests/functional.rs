use sns_reputation::{
    entrypoint::process_instruction,
    instruction::vote,
    state::{reputation_score::ReputationScore, Tag},
};
use solana_sdk::transaction::Transaction;
use vote::Params;

use {
    solana_program::pubkey::Pubkey,
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{
        account::Account,
        signer::{keypair::Keypair, Signer},
    },
};

pub mod common;

#[tokio::test]
async fn test_offer() {
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

    let instruction = vote(
        vote::Accounts {
            system_program: &Pubkey::default(),
            voter: &payer_pubkey,
            reputation_state_account: &reputation_state,
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

    assert_eq!(
        parsed,
        ReputationScore {
            nonce: reputation_state_nonce,
            upvote: 1,
            downvote: 0
        }
    );
}