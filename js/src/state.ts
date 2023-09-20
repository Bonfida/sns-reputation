import { deserialize, Schema } from 'borsh';
import { Connection, PublicKey } from '@solana/web3.js';

export enum Tag {
  Uninitialized = 0,
  ReputationScore = 1,
  UserVote = 2,
}

export class ReputationScoreState {
  static SEED = 'example_seed';
  tag: Tag;
  nonce: number;
  upvote: number;
  downvote: number;

  static schema = {
    struct: { tag: 'u64', nonce: 'u8', upvote: 'u64', downvote: 'u64' },
  };

  constructor(obj: { tag: Tag; nonce: number; upvote: bigint; downvote: bigint }) {
    this.tag = obj.tag;
    this.nonce = obj.nonce;
    this.upvote = Number(obj.upvote);
    this.downvote = Number(obj.downvote);
  }

  static deserialize(data: Buffer): ReputationScoreState {
    return new ReputationScoreState(deserialize(this.schema, data) as any);
  }

  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error('State account not found');
    }
    return this.deserialize(accountInfo.data);
  }
  static async findKey(programId: PublicKey, userAddress: PublicKey) {
    return await PublicKey.findProgramAddress(
      [userAddress.toBytes()],
      programId,
    );
  }
  static async findUserVoteKey(
    programId: PublicKey,
    userAddresses: [votee: PublicKey, voter: PublicKey],
  ) {
    return await PublicKey.findProgramAddress(
      userAddresses.map((a) => a.toBytes()),
      programId,
    );
  }
}
