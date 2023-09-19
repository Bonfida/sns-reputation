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

  const { upvote, downvote } = await ReputationScoreState.retrieve(connection, key);

  return upvote - downvote;
};
