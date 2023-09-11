import { Connection, PublicKey } from "@solana/web3.js";
import { SNS_REPUTATION_ID } from "./bindings";

/**
 * This function can be used to retrieve the EXAMPLE accounts of an owner
 * @param connection A solana RPC connection
 * @param owner The owner
 * @returns
 */
export const getForOwner = async (connection: Connection, owner: PublicKey) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "3", // Account Tag EXAMPLE
      },
    },
    {
      memcmp: {
        offset: 1,
        bytes: owner.toBase58(),
      },
    },
  ];

  const result = await connection.getProgramAccounts(SNS_REPUTATION_ID, {
    filters,
  });

  return result;
};
