//! Example instruction //TODO
use crate::error::SnsReputationError;
use bonfida_utils::checks::check_account_owner;
use solana_program::{lamports, program::invoke_signed, rent::Rent, sysvar::Sysvar};

use crate::state::{
    reputation_score::{self, ReputationScore},
    user_vote::UserVote,
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
    pub system_program: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub voter: &'a T,

    #[cons(writable)]
    pub reputation_state_account: &'a T,

    #[cons(writable)]
    pub user_vote_state_account: &'a T,
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
            user_vote_state_account: next_account_info(accounts_iter)?,
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

    let user_vote_key = UserVote::find_key(program_id, &(params.user_key, *accounts.voter.key));

    check_account_key(accounts.user_vote_state_account, &user_vote_key.0)?;

    let mut user_vote = if accounts.user_vote_state_account.data_is_empty() {
        let space = UserVote::default().borsh_len() + std::mem::size_of::<Tag>();
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(space);

        // Allocate account + set nonce
        invoke_signed(
            &solana_program::system_instruction::create_account(
                accounts.voter.key,
                accounts.user_vote_state_account.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[
                accounts.system_program.clone(),
                accounts.voter.clone(),
                accounts.user_vote_state_account.clone(),
            ],
            &[&[params.user_key.as_ref()]],
        )?;

        UserVote {
            value: params.is_upvote,
        }
    } else {
        // Deserialize the data into UserVote struct
        UserVote::from_buffer(
            &accounts.user_vote_state_account.data.borrow(),
            Tag::UserVote,
        )?
    };

    if user_vote.value == params.is_upvote {
        // User votes same value, throw error
        return Err(SnsReputationError::AlreadyVoted.into());
    } else {
        // The user has changed his vote
        user_vote.value = params.is_upvote;
        if params.is_upvote {
            reputation_score.upvote += 1;
            reputation_score.downvote -= 1;
        } else {
            reputation_score.downvote += 1;
            reputation_score.upvote -= 1;
        }
    }

    user_vote
        .save(&mut accounts.user_vote_state_account.data.borrow_mut())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    reputation_score
        .save(&mut accounts.reputation_state_account.data.borrow_mut())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
