import { beforeAll, expect, jest, test } from '@jest/globals';
import { Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { signAndSendTransactionInstructions, sleep } from './utils';
import {
  buildVotingInstruction,
  SNS_REPUTATION_ID_DEVNET,
} from '../src/bindings';
import {
  getReputationScore,
  getUserVoteAddress,
  getReputationScoreKey,
  getUserVote,
  getAllVotersForUser,
  getAllVoteesForVoter,
} from '../src/secondary_bindings';

let connection: Connection;

beforeAll(async () => {
  connection = new Connection(
    'https://explorer-api.devnet.solana.com/ ',
    'confirmed',
  );
});

jest.setTimeout(1_500_000);

const makeVote = async ({
  voter = undefined,
  vote = true,
  votee,
}: {
  voter?: Keypair;
  vote?: boolean;
  votee: Keypair;
}) => {
  if (!voter) {
    // Create new voter
    voter = Keypair.generate();
    // Airdrop some SOL
    const tx = await connection.requestAirdrop(
      voter.publicKey,
      LAMPORTS_PER_SOL,
    );
    await connection.confirmTransaction(tx, 'confirmed');
  }

  const [reputationScoreAddress] = await getReputationScoreKey(votee.publicKey);
  const [userVoteAddress] = await getUserVoteAddress({
    votee: votee.publicKey,
    voter: voter.publicKey,
  });

  const ix = buildVotingInstruction({
    programId: SNS_REPUTATION_ID_DEVNET,
    voter: voter.publicKey,
    userKey: votee.publicKey,
    userVotePdaAddress: userVoteAddress,
    reputationScorePdaAddress: reputationScoreAddress,
    isUpvote: vote,
  });

  await signAndSendTransactionInstructions(connection, [voter], voter, [ix]);

  return { voter };
};

const checkScore = (votee) => getReputationScore(connection, votee.publicKey);

/**
 * Test scenario
 *
 * 1. Check that initial score is 0
 * 2. Upvote, check that score is 1
 * 3. Downvote with the same user, check that score is -1
 * 4. Downvote with ANOTHER user, check that score is -2
 * 5. Check that voter can vote over another VOTEE and score is correct
 * 6. Check that we can retrieve current user vote and the valus is correct
 * 7. Check that "getAllVotersForUser" returns exactly 2 expected voters
 *
 */

test('Check voting flow', async () => {
  const votee = Keypair.generate();

  // Initial score should be 0
  expect(await checkScore(votee)).toEqual(0);

  // Make upvote
  const { voter } = await makeVote({ votee, vote: true });
  expect(await checkScore(votee)).toEqual(1);

  // Downvote with the same user
  await makeVote({ votee, vote: false, voter });
  // Now score should be -1, because same user changed his vote
  expect(await checkScore(votee)).toEqual(-1);

  // Make new downvote by NEW user
  const { voter: anotherVoter } = await makeVote({ votee, vote: false });
  // Now score should be -2, because two users downvoted
  expect(await checkScore(votee)).toEqual(-2);

  const anotherVotee = Keypair.generate();

  // Check that same voter can vote over another votee
  await makeVote({ votee: anotherVotee, vote: false, voter });
  expect(await checkScore(anotherVotee)).toEqual(-1);

  const voterVote = await getUserVote(connection, {
    votee: votee.publicKey,
    voter: voter.publicKey,
  });

  // voter's latest vote is -1, means "false"
  expect(voterVote).toEqual(false);

  const votersList = await getAllVotersForUser(connection, votee.publicKey);

  expect(votersList.length).toBe(2);
  // We're doing forEach because Solana might return the list in incorrect order
  // since we're making transactions one by one with no delay
  votersList.forEach(item => {
    expect([
      voter.publicKey.toBase58(),
      anotherVoter.publicKey.toBase58()
    ]).toContain(item.voter.toBase58());
  })

  const voteesList = await getAllVoteesForVoter(connection, voter.publicKey);

  expect(voteesList.length).toBe(2);
  voteesList.forEach(item => {
    expect([
      votee.publicKey.toBase58(),
      anotherVotee.publicKey.toBase58()
    ]).toContain(item.votee.toBase58());
  });
});
