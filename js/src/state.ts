import { deserialize, Schema } from 'borsh';
import { Connection, PublicKey } from '@solana/web3.js';

export enum Tag {
  Uninitialized = 0,
  ReputationScore = 1,
  UserVote = 2,
}

export class ReputationScoreState {
  tag: Tag;
  nonce: number;
  upvote: number;
  downvote: number;

  static schema = {
    struct: { tag: 'u64', nonce: 'u8', upvote: 'u64', downvote: 'u64' },
  };

  constructor(obj: {
    tag: Tag;
    nonce: number;
    upvote: bigint;
    downvote: bigint;
  }) {
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
}

export interface UserVote {
  tag: Tag;
  value: boolean;
  votee: PublicKey;
  voter: PublicKey;
}

export class UserVoteState implements UserVote {
  tag: Tag;
  value: boolean;
  votee: PublicKey;
  voter: PublicKey;

  static schema = {
    struct: {
      tag: 'u64',
      value: 'u8',
      votee: { array: { type: 'u8', len: 32 } },
      voter: { array: { type: 'u8', len: 32 } },
    },
  };

  constructor(obj: {
    tag: Tag;
    value: boolean;
    votee: Uint8Array;
    voter: Uint8Array;
  }) {
    this.tag = obj.tag;
    this.value = obj.value;
    this.votee = new PublicKey(obj.votee);
    this.voter = new PublicKey(obj.voter);
  }

  static deserialize(data: Buffer): UserVoteState {
    return new UserVoteState(deserialize(this.schema, data) as any);
  }

  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error('State account not found');
    }
    return this.deserialize(accountInfo.data);
  }
  static async findKey(
    programId: PublicKey,
    { votee, voter }: { votee: PublicKey; voter: PublicKey },
  ) {
    return await PublicKey.findProgramAddress(
      [votee.toBytes(), voter.toBytes()],
      programId,
    );
  }
}
