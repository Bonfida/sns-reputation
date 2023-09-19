import { deserialize, Schema } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";
import BN from "bn.js";

export enum Tag {
  Uninitialized = 0,
  ReputationScore = 1,
  UserVote = 2,
}

export class ReputationScoreState {
  static SEED = "example_seed";
  tag: Tag;
  nonce: number;
  upvote: number;
  downvote: number;

  static schema: Schema = new Map([
    [
      ReputationScoreState,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["nonce", "u8"],
          ["upvote", "u64"],
          ["downvote", "u64"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: BN;
    nonce: number;
    upvote: BN;
    downvote: BN;
  }) {
    this.tag = obj.tag.toNumber() as Tag;
    this.nonce = obj.nonce;
    this.upvote = obj.upvote.toNumber();
    this.downvote = obj.downvote.toNumber();
  }

  static deserialize(data: Buffer): ReputationScoreState {
    return deserialize(this.schema, ReputationScoreState, data);
  }

  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("State account not found");
    }
    return this.deserialize(accountInfo.data);
  }
  static async findKey(programId: PublicKey, userAddress: PublicKey) {
    return await PublicKey.findProgramAddress(
      [userAddress.toBytes()],
      programId
    );
  }
}
