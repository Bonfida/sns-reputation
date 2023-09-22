import { Connection, PublicKey } from "@solana/web3.js";
import { SNS_REPUTATION_ID_DEVNET } from "./bindings";
import { ReputationScoreState, UserVoteState } from './state';

export const getReputationScoreKey = (user: PublicKey) => {
  return ReputationScoreState.findKey(SNS_REPUTATION_ID_DEVNET, user);
}

/**
 * Retrieve user reputation score, based on number of upvotes and downvotes.
 * @param connection – A solana RPC connection
 * @param votee – User voted over by other users
 * @returns reputation score
 * @example
 *
 * const score = getReputationScore(connection, votee.publicKey);
 */
export const getReputationScore = async (connection: Connection, votee: PublicKey): Promise<number> => {
  const [key] = await getReputationScoreKey(votee);

  let upvote = 0;
  let downvote = 0;

  try {
    const result = await ReputationScoreState.retrieve(connection, key);

    upvote = result.upvote;
    downvote = result.downvote;
  } catch (err: any) {
    if (!(err instanceof Error)) {
      throw err
    }
  }

  return upvote - downvote;
};

export const getUserVoteAddress = (addresses: Parameters<typeof UserVoteState.findKey>[1]) => {
  return UserVoteState.findKey(SNS_REPUTATION_ID_DEVNET, addresses);
}
/**
 * Retrieve user vote.
 *
 * @param connection – A solana RPC connection
 * @param users – User voted over by other users
 * @returns reputation score
 * @example
 *
 * const score = getReputationScore(connection, votee.publicKey);
 */
export const getUserVote = async (
  connection: Connection,
  users: Parameters<typeof UserVoteState.findKey>[1],
): Promise<number> => {
  const [key] = await getUserVoteAddress(users);

  console.log('key', key.toBase58())

  let upvote = 0;
  let downvote = 0;

  try {
    const result = await UserVoteState.retrieve(connection, key);

    console.log('result', result);
  } catch (err) {
    console.log('err', err);
    if (!(err instanceof Error)) {
      throw err
    }
  }

  return upvote - downvote;
};
