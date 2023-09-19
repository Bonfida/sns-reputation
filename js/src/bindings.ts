import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { voteInstruction } from "./raw_instructions";

/**
 * Mainnet program ID
 */
export const SNS_REPUTATION_ID = new PublicKey(""); //TODO

/**
 * Devnet program ID (might not have the latest version deployed!)
 */
export const SNS_REPUTATION_ID_DEVNET = new PublicKey("HVFVK2UComnzuLfDbPukyt86LGi51iLQUL3aGBEVqLni"); //TODO

/**
 * This function can be used as a js binding example.
 * @param feePayer The fee payer of the transaction
 * @param programId The program ID
 * @returns
 */
export const vote = async (
  feePayer: PublicKey,
  programId: PublicKey,
  userKey: PublicKey,
  isUpvote: boolean,
  userVote: PublicKey
) => {
  const ix = new voteInstruction({
    userKey: userKey.toBytes(),
    isUpvote,
  }).getInstruction(
    programId,
    SystemProgram.programId,
    SYSVAR_RENT_PUBKEY,
    feePayer,
    userVote
  );

  return [ix];
};
