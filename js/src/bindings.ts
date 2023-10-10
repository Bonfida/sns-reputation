import { PublicKey, SystemProgram } from "@solana/web3.js";
import { voteInstruction } from "./raw_instructions";
import { VoteValue } from "./state";

/**
 * Mainnet program ID
 */
export const SNS_REPUTATION_ID = new PublicKey(
  "4X9mF1yUx2ez6ifYCmr2aYJnX5DkKAxbu5QD93s7gooG"
);

/**
 * Devnet program ID (might not have the latest version deployed!)
 */
export const SNS_REPUTATION_ID_DEVNET = new PublicKey(
  "HVFVK2UComnzuLfDbPukyt86LGi51iLQUL3aGBEVqLni"
);

interface VotingInstructionParams {
  programId: PublicKey;
  voter: PublicKey;
  userKey: PublicKey;
  userVotePdaAddress: PublicKey;
  reputationScorePdaAddress: PublicKey;
  voteValue: VoteValue;
  voterStakeAddress?: PublicKey;
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
 * @param params.voteValue - New voter's vote (see VoteValue type).
 * @returns A promise that resolves when the vote is successfully cast.
 */
export const buildVotingInstruction = ({
  programId,
  voter,
  userKey,
  userVotePdaAddress,
  reputationScorePdaAddress,
  voterStakeAddress,
  voteValue,
}: VotingInstructionParams) => {
  return new voteInstruction({
    userKey: userKey.toBytes(),
    voteValue,
  }).getInstruction(
    programId,
    SystemProgram.programId,
    voter,
    reputationScorePdaAddress,
    userVotePdaAddress,
    voterStakeAddress
  );
};
