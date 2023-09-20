import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import { voteInstruction } from './raw_instructions';

/**
 * Mainnet program ID
 */
// export const SNS_REPUTATION_ID = new PublicKey(""); //TODO:

/**
 * Devnet program ID (might not have the latest version deployed!)
 */
export const SNS_REPUTATION_ID_DEVNET = new PublicKey(
  'HVFVK2UComnzuLfDbPukyt86LGi51iLQUL3aGBEVqLni',
);

interface VotingInstructionParams {
  programId: PublicKey;
  voter: PublicKey;
  userKey: PublicKey;
  userVotePdaAddress: PublicKey;
  reputationScorePdaAddress: PublicKey;
  isUpvote: boolean;
}
/**
 * Creates voting instruction.
 *
 * @param params - The parameters for the vote function.
 * @param params.programId - The program ID.
 * @param params.voter - Voter, and the actual fee payer of the transaction.
 * @param params.userKey - The votee account.
 * @param params.userVotePdaAddress - PDA: previous voter's vote state.
 * @param params.reputationScorePdaAddress - PDA: votee reputation score.
 * @param params.isUpvote - New voter's vote (true for upvote, false for downvote).
 * @returns A promise that resolves when the vote is successfully cast.
 */
export const buildVotingInstruction = ({
  programId,
  voter,
  userKey,
  userVotePdaAddress,
  reputationScorePdaAddress,
  isUpvote,
}: VotingInstructionParams) => {
  return new voteInstruction({
    userKey: userKey.toBytes(),
    isUpvote,
  }).getInstruction(
    programId,
    SystemProgram.programId,
    voter,
    reputationScorePdaAddress,
    userVotePdaAddress,
  );
};
