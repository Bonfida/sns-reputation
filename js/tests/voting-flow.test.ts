import { beforeAll, expect, jest, test } from "@jest/globals";
import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { signAndSendTransactionInstructions, sleep } from "./utils";
import {
  buildVotingInstruction,
  SNS_REPUTATION_ID_DEVNET,
} from "../src/bindings";
import {
  getReputationScore,
  getUserVoteAddress,
  getReputationScoreKey,
  getUserVote,
  getAllVotersForUser,
  getAllVoteesForVoter,
} from "../src/secondary_bindings";
import { VoteValue } from "../src/state";

let connection: Connection;

beforeAll(async () => {
  connection = new Connection(
    "https://explorer-api.devnet.solana.com/ ",
    "confirmed"
  );
});

jest.setTimeout(1_500_000);

const makeVote = async ({
  voter = undefined,
  vote,
  votee,
}: {
  voter?: Keypair;
  vote: VoteValue;
  votee: Keypair;
}) => {
  if (!voter) {
    // Create new voter
    voter = Keypair.generate();
    // Airdrop some SOL
    const tx = await connection.requestAirdrop(
      voter.publicKey,
      LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(tx, "confirmed");
  }

  const [reputationScoreAddress] = await getReputationScoreKey(
    votee.publicKey,
    SNS_REPUTATION_ID_DEVNET
  );
  const [userVoteAddress] = await getUserVoteAddress(
    {
      votee: votee.publicKey,
      voter: voter.publicKey,
    },
    SNS_REPUTATION_ID_DEVNET
  );

  const ix = buildVotingInstruction({
    programId: SNS_REPUTATION_ID_DEVNET,
    voter: voter.publicKey,
    userKey: votee.publicKey,
    userVotePdaAddress: userVoteAddress,
    reputationScorePdaAddress: reputationScoreAddress,
    voteValue: vote,
  });

  await signAndSendTransactionInstructions(connection, [voter], voter, [ix]);

  return { voter };
};

const checkScore = (votee) =>
  getReputationScore(connection, votee.publicKey, SNS_REPUTATION_ID_DEVNET);

/**
 * Test scenario
 *
 * 1. Check that initial score of voteeA is 0
 * 2. Upvote, check that voteeA's score is 1
 * 3. Downvote voteeA, check that score is -1
 * 4. Downvote voteeA with ANOTHER user, check that score is -2
 * 5. Check that voter can vote over another voteeB and score is correct
 * 6. Check that we can fetch voter's vote and the value is correct
 * 7. Check that "getAllVotersForUser" returns exactly 2 expected voters
 * 8. With the same voter try to downvote voteeB user again, to actually "undo"
 *    previos vote, and check that now voteeB's score is 0
 * 9. Check that after "undo" operation voter is not associated with voteeB anympre
 */

test("Check voting flow", async () => {
  const voteeA = Keypair.generate();

  // Initial score should be 0
  expect(await checkScore(voteeA)).toEqual(0);

  // Make upvote
  const { voter } = await makeVote({ votee: voteeA, vote: VoteValue.Upvote });
  expect(await checkScore(voteeA)).toEqual(1);

  // Downvote with the same user
  await makeVote({ votee: voteeA, vote: VoteValue.Downvote, voter });
  // Now score should be -1, because same user changed his vote
  expect(await checkScore(voteeA)).toEqual(-1);

  // Make new downvote by NEW user
  const { voter: anotherVoter } = await makeVote({
    votee: voteeA,
    vote: VoteValue.Downvote,
  });
  // Now score should be -2, because two users downvoted
  expect(await checkScore(voteeA)).toEqual(-2);

  const voteeB = Keypair.generate();

  // Check that same voter can vote over another votee
  await makeVote({ votee: voteeB, vote: VoteValue.Downvote, voter });
  expect(await checkScore(voteeB)).toEqual(-1);

  const voterVote = await getUserVote(
    connection,
    {
      votee: voteeA.publicKey,
      voter: voter.publicKey,
    },
    SNS_REPUTATION_ID_DEVNET
  );

  expect(voterVote).toEqual(VoteValue.Downvote);

  const votersList = await getAllVotersForUser(
    connection,
    voteeA.publicKey,
    SNS_REPUTATION_ID_DEVNET
  );

  expect(votersList.length).toBe(2);
  // We're doing forEach because Solana might return the list in incorrect order
  // since we're making transactions one by one with no delay
  votersList.forEach((item) => {
    expect([
      voter.publicKey.toBase58(),
      anotherVoter.publicKey.toBase58(),
    ]).toContain(item.voter.toBase58());
  });

  let voteesList = await getAllVoteesForVoter(
    connection,
    voter.publicKey,
    SNS_REPUTATION_ID_DEVNET
  );

  expect(voteesList.length).toBe(2);
  voteesList.forEach((item) => {
    expect([
      voteeA.publicKey.toBase58(),
      voteeB.publicKey.toBase58(),
    ]).toContain(item.votee.toBase58());
  });

  // Test undo voting
  await makeVote({ votee: voteeB, vote: VoteValue.NoVote, voter });
  // Now score should be 0, because same user undo his vote
  expect(await checkScore(voteeB)).toEqual(0);

  // Check that now "voter" is not associated with "voteeB"
  voteesList = await getAllVoteesForVoter(
    connection,
    voter.publicKey,
    SNS_REPUTATION_ID_DEVNET
  );
  expect(voteesList.length).toBe(1);
  voteesList.forEach((item) => {
    expect([voteeB.publicKey.toBase58()]).not.toContain(item.votee.toBase58());
  });
});
