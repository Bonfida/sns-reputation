import { Connection, PublicKey, StakeProgram } from "@solana/web3.js";
import { SNS_REPUTATION_ID } from "./bindings";
import { ReputationScoreState, UserVoteState, VoteValue } from "./state";
import base58 from "bs58";
import BN from "bn.js";
import { Buffer } from "buffer";

export const getReputationScoreKey = (
  user: PublicKey,
  programId = SNS_REPUTATION_ID
) => {
  return ReputationScoreState.findKey(programId, user);
};

/**
 * Retrieve user reputation score, based on number of upvotes and downvotes.
 * @param connection - A solana RPC connection
 * @param votee - User voted over by other users
 * @returns reputation score
 * @example
 *
 * const score = getReputationScore(connection, votee.publicKey);
 */
export const getReputationScore = async (
  connection: Connection,
  votee: PublicKey,
  programId = SNS_REPUTATION_ID
): Promise<bigint> => {
  const [key] = await getReputationScoreKey(votee, programId);

  let upvote = BigInt(0);
  let downvote = BigInt(0);

  try {
    const result = await ReputationScoreState.retrieve(connection, key);

    upvote = result.upvote;
    downvote = result.downvote;
  } catch (err: any) {
    if (!(err instanceof Error)) {
      throw err;
    }
  }

  return upvote - downvote;
};

export const getUserVoteAddress = (
  addresses: Parameters<typeof UserVoteState.findKey>[1],
  programId = SNS_REPUTATION_ID
) => {
  return UserVoteState.findKey(programId, addresses);
};
/**
 * Retrieve user vote.
 *
 * @param connection - A solana RPC connection
 * @param users - Votee and voter addresses to derive correct vote
 * @returns current user vote
 * @example
 *
 * const userVote = getUserVote(connection, { votee: votee.publicKey, voter: voter.publicKey });
 */
export const getUserVote = async (
  connection: Connection,
  users: Parameters<typeof getUserVoteAddress>[0],
  programId = SNS_REPUTATION_ID
): Promise<bigint | null> => {
  const [key] = await getUserVoteAddress(users, programId);

  try {
    const result = await UserVoteState.retrieve(connection, key);

    return result.value;
  } catch (err) {
    if (!(err instanceof Error)) {
      throw err;
    }
  }

  return null;
};

/**
 * Returns all voters that voted over asked votee
 *
 * @param connection - A solana RPC connection
 * @param votee - User for whom we are looking for all voters
 * @returns voters that voter over votee
 * @example
 * const voters = getAllVotersForUser(connection, votee.publicKey);
 */
export const getAllVotersForUser = async (
  connection: Connection,
  votee: PublicKey,
  programId = SNS_REPUTATION_ID
): Promise<UserVoteState[]> => {
  try {
    const filters = [
      {
        // tag + voteValue + votee pubkey + voter pubkey
        dataSize: 8 + 1 + 32 + 32,
      },
      {
        memcmp: {
          offset: 8 + 1, // tag + voteValue
          bytes: votee.toBase58(),
        },
      },
    ];

    const result = await connection.getProgramAccounts(programId, {
      filters,
    });

    return result.map((item) => UserVoteState.deserialize(item.account.data));
  } catch (err) {
    console.error(err);
    return [];
  }
};

export const getAllVoteesForVoter = async (
  connection: Connection,
  voter: PublicKey,
  programId = SNS_REPUTATION_ID
): Promise<UserVoteState[]> => {
  try {
    const filters = [
      {
        // tag + voteValue + votee pubkey + voter pubkey
        dataSize: 8 + 1 + 32 + 32,
      },
      {
        memcmp: {
          // tag + voteValue + votee pubkey
          offset: 8 + 1 + 32,
          bytes: voter.toBase58(),
        },
      },
    ];

    const result = await connection.getProgramAccounts(programId, {
      filters,
    });

    return result.map((item) => UserVoteState.deserialize(item.account.data));
  } catch (err) {
    console.error(err);
    return [];
  }
};

export const getBestStakeAccountForVoter = async (
  connection: Connection,
  voter: PublicKey,
  max_activation_epoch: number
): Promise<PublicKey[] | undefined> => {
  let expected_tag_filter = {
    offset: 0,
    bytes: base58.encode([2, 0, 0, 0]),
  };
  let expected_stake_owner_filter = {
    offset: 12,
    bytes: voter.toBase58(),
  };
  let candidates = await connection.getProgramAccounts(StakeProgram.programId, {
    filters: [
      { memcmp: expected_tag_filter },
      { memcmp: expected_stake_owner_filter },
    ],
  });
  let result = candidates.map((a) => {
    let stake_info = parseStakeAndEpoch(a.account.data);
    return { account: a, stake_info };
  });
  result = result.filter(
    (a) => a.stake_info.activation_epoch <= max_activation_epoch
  );
  result = result.sort((a, b) => {
    return a.stake_info.stake - b.stake_info.stake;
  });

  return result.map((a) => a.account.pubkey);
};

export const parseStakeAndEpoch = (data: Buffer) => {
  let stake_index = 12 + 32 + 32 + 8 + 8 + 32 + 32;
  // stake_index = 4 + 8 + 32 + 32 + 8 + 8 + 32 + 32;
  let epoch_index = stake_index + 8;
  let activation_epoch = new BN(
    data.slice(epoch_index, epoch_index + 8),
    undefined,
    "le"
  ).toNumber();
  let stake = new BN(
    data.slice(stake_index, stake_index + 8),
    undefined,
    "le"
  ).toNumber();
  return { stake, activation_epoch };
};
