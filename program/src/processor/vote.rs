//! Example instruction //TODO

use bonfida_utils::checks::check_account_owner;

use crate::state::{reputation_score::ReputationScore, Tag};

use {
    bonfida_utils::{
        checks::{check_account_key, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// An example input parameter
    pub user_key: Pubkey,
    pub is_upvote: bool,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub voter: &'a T,

    #[cons(writable)]
    pub reputation_state_account: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            voter: next_account_info(accounts_iter)?,
            reputation_state_account: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;

        // Check signer
        check_signer(accounts.voter)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    // Verify the example state account
    let (reputation_score_key, reputation_score_nonce) =
        ReputationScore::find_key(program_id, &params.user_key);

    check_account_key(accounts.reputation_state_account, &reputation_score_key)?;

    // TODO: save

    if accounts.reputation_state_account.data_is_empty() {
        // Allocate account + set nonce
    }

    let data = ReputationScore::from_buffer(
        &accounts.reputation_state_account.data.borrow(),
        Tag::ReputationScore,
    )?;

    Ok(())
}
