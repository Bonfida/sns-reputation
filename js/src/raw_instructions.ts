// This file is auto-generated. DO NOT EDIT
import BN from 'bn.js';
import { serialize } from 'borsh';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';

export interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}
export class voteInstruction {
  tag: number;
  userKey: Uint8Array;
  voteValue: number;

  static schema = {
    struct: {
      tag: 'u8',
      userKey: { array: { type: 'u8', len: 32 } },
      voteValue: 'u8',
    },
  };

  constructor(obj: { userKey: Uint8Array; voteValue: number }) {
    this.tag = 0;
    this.userKey = obj.userKey;
    this.voteValue = obj.voteValue;
  }
  serialize(): Uint8Array {
    return serialize(voteInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    systemProgram: PublicKey,
    voter: PublicKey,
    reputationStateAccount: PublicKey,
    userVoteStateAccount: PublicKey,
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: voter,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: reputationStateAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: userVoteStateAccount,
      isSigner: false,
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
