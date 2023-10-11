//! Instruction for storing and processing all votes over the votee account by using
//! two separate PDAs:
//! 1. Reputation score PDA - accumulates all voters' votes over the votee account.
//! 2. User vote PDA – stores voter's vote.

use bonfida_utils::checks::check_account_owner;
use solana_program::msg;
use solana_program::stake::state::StakeState;
use solana_program::{program::invoke_signed, rent::Rent, sysvar::Sysvar};

use crate::error::SnsReputationError;
use crate::state::{reputation_score::ReputationScore, user_vote::UserVote, Tag, VoteValue};

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

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
pub struct Params {
    /// votee account pubkey
    pub user_key: Pubkey,
    /// voter's vote
    pub vote_value: VoteValue,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    pub system_program: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub voter: &'a T,

    /// PDA for storing ReputationScore data
    #[cons(writable)]
    pub reputation_state_account: &'a T,

    /// PDA that stores voter's vote, that is derived from votee and vote's keys
    #[cons(writable)]
    pub user_vote_state_account: &'a T,

    /// Stake account associated with the voter
    pub voter_stake_accounts: &'a [T],
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            voter: next_account_info(accounts_iter)?,
            reputation_state_account: next_account_info(accounts_iter)?,
            user_vote_state_account: next_account_info(accounts_iter)?,
            voter_stake_accounts: accounts_iter.as_slice(),
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

    // Check that voter is authorized to vote
    #[cfg(not(feature = "devnet"))]
    let vote_weight = if params.vote_value != VoteValue::NoVote {
        if accounts.voter_stake_accounts.is_empty() {
            return Err(SnsReputationError::MissingStakeAccount.into());
        }
        let mut total_stake = 0;
        for voter_stake_account in accounts.voter_stake_accounts.iter() {
            check_account_owner(voter_stake_account, &solana_program::stake::program::ID)?;
            let parsed_stake = solana_program::stake::state::StakeState::deserialize(
                &mut (&voter_stake_account.data.borrow() as &[u8]),
            )?;
            total_stake += if let StakeState::Stake(meta, stake) = parsed_stake {
                if &meta.authorized.staker != accounts.voter.key {
                    msg!("The staking account should be owned by the voter");
                    return Err(SnsReputationError::InvalidStakeAccount.into());
                }
                let clock = solana_program::sysvar::clock::Clock::get()?;
                if clock
                    .epoch
                    .checked_sub(stake.delegation.activation_epoch)
                    .unwrap_or_default()
                    < 2
                // At least three days lockup
                {
                    msg!("Funds have not been staked for long enough.");
                    return Err(SnsReputationError::InvalidStakeAccount.into());
                }
                stake.delegation.stake as i64
            } else {
                return Err(SnsReputationError::InvalidStakeAccount.into());
            }
        }
        total_stake
    } else {
        0
    };
    #[cfg(feature = "devnet")]
    let vote_weight = 1;

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

    let (user_vote_key, use_key_nonce) =
        UserVote::find_key(program_id, &(params.user_key, *accounts.voter.key));

    check_account_key(accounts.user_vote_state_account, &user_vote_key)?;

    let new_vote_value = (params.vote_value as i64).checked_mul(vote_weight).unwrap();

    let user_vote = if accounts.user_vote_state_account.data_is_empty() {
        // If UserVote PDA is empty, means we're dealing with the initial user's vote
        // Create UserVote PDA and update initial ReputationScore value

        // Throw an error if user's initial vote is VoteValue::NoVote, because this
        // value is used to "undo" user's vote and get their rent back, so it makes
        // no sense to process this value for the initial vote
        if params.vote_value == VoteValue::NoVote {
            return Err(SnsReputationError::NoVoteExists.into());
        }

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
            // Seeds (votee + voter) to derive PDA
            &[&[
                params.user_key.as_ref(),
                accounts.voter.key.as_ref(),
                &[use_key_nonce],
            ]],
        )?;

        let vote = UserVote {
            value: new_vote_value,
            votee: params.user_key,
            voter: *accounts.voter.key,
        };

        if params.vote_value == VoteValue::Upvote {
            reputation_score.upvote = reputation_score
                .upvote
                .checked_add(new_vote_value.unsigned_abs())
                .unwrap();
        } else if params.vote_value == VoteValue::Downvote {
            reputation_score.downvote = reputation_score
                .downvote
                .checked_add(new_vote_value.unsigned_abs())
                .unwrap();
        }

        vote
    } else {
        // Otherwise, derive UserVote value and update the ReputationScore
        // value correspondingly

        let mut vote = UserVote::from_buffer(
            &accounts.user_vote_state_account.data.borrow(),
            Tag::UserVote,
        )?;

        // Return an error if user voted with the same value
        if (vote.value.signum()) == ((params.vote_value as i64).signum()) {
            return Err(SnsReputationError::AlreadyVoted.into());
        }

        // If user voted with VoteValue::NoVote, it means that the user wants to undo their previous vote
        if params.vote_value == VoteValue::NoVote {
            let lamports = **accounts.user_vote_state_account.lamports.borrow_mut();
            **accounts.user_vote_state_account.lamports.borrow_mut() = 0;
            **accounts.voter.lamports.borrow_mut() += lamports;

            if vote.value.signum() == (VoteValue::Upvote as i64).signum() {
                reputation_score.upvote = reputation_score
                    .upvote
                    .checked_sub(vote.value.unsigned_abs())
                    .unwrap();
            } else if vote.value.signum() == (VoteValue::Downvote as i64).signum() {
                reputation_score.downvote = reputation_score
                    .downvote
                    .checked_sub(vote.value.unsigned_abs())
                    .unwrap();
            }

            reputation_score
                .save(&mut accounts.reputation_state_account.data.borrow_mut())
                .map_err(|_| ProgramError::InvalidAccountData)?;

            return Ok(());
        }

        // The user has changed their vote
        match params.vote_value {
            VoteValue::Upvote => {
                reputation_score.upvote = reputation_score
                    .upvote
                    .checked_add(new_vote_value.unsigned_abs())
                    .unwrap();
                reputation_score.downvote = reputation_score
                    .downvote
                    .checked_sub(vote.value.unsigned_abs())
                    .unwrap();
            }
            VoteValue::Downvote => {
                reputation_score.downvote = reputation_score
                    .downvote
                    .checked_add(new_vote_value.unsigned_abs())
                    .unwrap();
                reputation_score.upvote = reputation_score
                    .upvote
                    .checked_sub(vote.value.unsigned_abs())
                    .unwrap();
            }
            _ => {}
        }

        vote.value = new_vote_value;

        vote
    };

    user_vote
        .save(&mut accounts.user_vote_state_account.data.borrow_mut())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    reputation_score
        .save(&mut accounts.reputation_state_account.data.borrow_mut())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}
