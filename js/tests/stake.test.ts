import { Connection, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  getBestStakeAccountForVoter,
  parseStakeAndEpoch,
} from "../src/secondary_bindings";

let connection: Connection;

beforeAll(async () => {
  connection = new Connection(
    "https://api.mainnet-beta.solana.com/ ",
    "confirmed"
  );
});

test("test stake account retrieval", async () => {
  let accounts = await getBestStakeAccountForVoter(
    connection,
    new PublicKey("J6QDztZCegYTWnGUYtjqVS9d7AZoS43UbEQmMcdGeP5s"),
    1000
  );
  expect(accounts?.length).toBeGreaterThan(0);
  if (!accounts) {
    return;
  }
  console.log(accounts.length);
  console.log(accounts[0].toBase58());
  let account = await connection.getAccountInfo(accounts[0]);
  expect(account).toBeDefined();
  if (!account) {
    return;
  }
  let { stake, activation_epoch } = await parseStakeAndEpoch(account.data);
  expect(stake).toBe(LAMPORTS_PER_SOL * 0.1);
  expect(activation_epoch).toBe(513);
});
