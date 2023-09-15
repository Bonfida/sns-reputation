//! Example instruction //TODO

use bonfida_utils::checks::check_account_owner;
use solana_program::{lamports, program::invoke_signed, rent::Rent, sysvar::Sysvar};

use crate::state::{
    reputation_score::{self, ReputationScore},
    Tag,
};

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
    /// votee account pubkey
    pub user_key: Pubkey,
    pub is_upvote: bool,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    /// This account is required so we can use it to make manipulations like
    /// invoke_signed and etc, where system program is required as an argument
    pub system_program: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub voter: &'a T,

    #[cons(writable)]
    /// Why we need to pass that account, if we anyway can get access to
    /// ReputationScore from votee account that is passed via params? So we will
    /// be able to mutate it?
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

    // We have voter account, who pays and votes
    // We have votee account, for whom voter votes
    // We have PDA where we store ReputationScore
    // What is reputation_state_account? It's a PDA?

    // In find_key + check_account_key we're checking that votee reputation_score_key is the same
    // as reputation_state_account that is passed to the instruction. We just
    // validate that we're making changes on the correct votee
    let (reputation_score_key, reputation_score_nonce) =
        ReputationScore::find_key(program_id, &params.user_key);

    check_account_key(accounts.reputation_state_account, &reputation_score_key)?;

    let mut reputation_score = if accounts.reputation_state_account.data_is_empty() {
        let space = ReputationScore::default().borsh_len() + std::mem::size_of::<Tag>();
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(space);

        // Allocate account + set nonce
        invoke_signed(
            &solana_program::system_instruction::create_account(
                accounts.voter.key,
                accounts.reputation_state_account.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[
                accounts.system_program.clone(),
                accounts.voter.clone(),
                accounts.reputation_state_account.clone(),
            ],
            &[&[params.user_key.as_ref(), &[reputation_score_nonce]]],
        )?;

        ReputationScore {
            nonce: reputation_score_nonce,
            upvote: 0,
            downvote: 0,
        }
    } else {
        ReputationScore::from_buffer(
            &accounts.reputation_state_account.data.borrow(),
            Tag::ReputationScore,
        )?
    };

    // Why voter pays fee for PDA creating, if PDA will contain data of multiple
    // voters? It means that the first voter will pay for everyone?

    // Dunno how to derive previous voter's data, since I don't see any place where
    // we use voter's data except of the first voter who creates PDA for storing score
    // for all future votes

    if params.is_upvote {
        reputation_score.upvote += 1;
    } else {
        reputation_score.downvote += 1;
    }

    reputation_score
        .save(&mut accounts.reputation_state_account.data.borrow_mut())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
