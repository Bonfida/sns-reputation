import { Connection, PublicKey } from "@solana/web3.js";
import { SNS_REPUTATION_ID_DEVNET } from "./bindings";
import { ReputationScoreState } from './state';

/**
 * TODO:
 * This function can be used to retrieve the EXAMPLE accounts of an owner
 * @param connection A solana RPC connection
 * @param owner The owner
 * @returns
 */
export const getReputationScore = async (connection: Connection, user: PublicKey): Promise<number> => {
  const [key] = await ReputationScoreState.findKey(SNS_REPUTATION_ID_DEVNET, user);

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

export const getReputationScoreKey = (user: PublicKey) => {
  return ReputationScoreState.findKey(SNS_REPUTATION_ID_DEVNET, user);
}

export const getUserVoteAddress = (addresses: [votee: PublicKey, voter: PublicKey]) => {
  return ReputationScoreState.findUserVoteKey(SNS_REPUTATION_ID_DEVNET, addresses);
}
