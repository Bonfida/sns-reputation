import { Hono } from 'hono';
import { z, ZodError } from 'zod';
import { TTL, generateMessage, getCurrentTime, pubkey, verifyMessage } from './utils';
import { Connection, PublicKey } from '@solana/web3.js';

type Bindings = {
	DB: D1Database;

	NONCE_KV: KVNamespace;

	RPC_URL: string;
};

const app = new Hono<{ Bindings: Bindings }>();

const NonceRequest = z.object({
	userKey: pubkey,
});

app.post('/nonce', async (c) => {
	try {
		const json = await c.req.json();
		const { userKey } = NonceRequest.parse(json);

		const value = await c.env.NONCE_KV.get(userKey);

		if (!!value) {
			return c.json('Already generated nonce - wait', 400);
		}

		const msg = generateMessage(userKey);
		await c.env.NONCE_KV.put(userKey, msg, { expirationTtl: TTL });

		return c.json({ msg });
	} catch (err) {
		if (err instanceof ZodError) {
			return c.json('Bad request', 400);
		}
		return c.json('Error', 500);
	}
});

const ReportTxRquest = z.object({
	tx: z.string(),
	msgSig: z.string(),
	userKey: pubkey,
});

app.post('/report-tx', async (c) => {
	try {
		const json = await c.req.json();
		const { tx, msgSig, userKey } = ReportTxRquest.parse(json);

		const msg = await c.env.NONCE_KV.get(userKey);

		if (!msg) {
			return c.json('Nonce not found', 400);
		}

		const validSig = verifyMessage(msg, msgSig, userKey);
		if (!validSig) {
			return c.json('Invalid signature', 400);
		}
		await c.env.NONCE_KV.delete(userKey);

		const connection = new Connection(c.env.RPC_URL);

		const transaction = await connection.getTransaction(tx, { maxSupportedTransactionVersion: 1, commitment: 'confirmed' });

		if (transaction === null) {
			return c.json('Transaction not found', 400);
		}

		const isUserInvolved = !!transaction?.transaction.message.staticAccountKeys.some((e) => e.toBase58() === userKey);

		const existing = await c.env.DB.prepare('SELECT COUNT(*) as total FROM report_tx WHERE tx_sig = ?1 AND reported_by = ?2')
			.bind(tx, userKey)
			.first('total');

		if (Number(existing) !== 0) {
			return c.json('Already reported', 400);
		}

		const stmt = c.env.DB.prepare(
			`INSERT INTO 
			 report_tx (tx_sig, slot, slot_time, successful, reported_time, reported_by, reporter_involved)
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)`
		).bind(
			tx,
			transaction?.slot,
			transaction?.blockTime,
			Number(transaction?.meta?.err === null),
			getCurrentTime(),
			userKey,
			Number(isUserInvolved)
		);
		await stmt.run();

		return c.json('Success');
	} catch (err) {
		console.log(err);
		if (err instanceof ZodError) {
			return c.json('Bad request', 400);
		}
		return c.json('Error', 500);
	}
});

const ReportKeyRquest = z.object({
	key: pubkey,
	msgSig: z.string(),
	userKey: pubkey,
});

app.post('/report-key', async (c) => {
	try {
		const json = await c.req.json();
		const { key, msgSig, userKey } = ReportKeyRquest.parse(json);
		const msg = await c.env.NONCE_KV.get(userKey);

		if (!msg) {
			return c.json('Nonce not found', 400);
		}

		const validSig = verifyMessage(msg, msgSig, userKey);
		if (!validSig) {
			return c.json('Invalid signature', 400);
		}
		await c.env.NONCE_KV.delete(userKey);

		const connection = new Connection(c.env.RPC_URL);

		const info = await connection.getAccountInfo(new PublicKey(key));

		const existing = await c.env.DB.prepare('SELECT COUNT(*) as total FROM report_key WHERE key = ?1 AND reported_by = ?2')
			.bind(key, userKey)
			.first('total');

		if (Number(existing) !== 0) {
			return c.json('Already reported', 400);
		}

		const stmt = c.env.DB.prepare(
			`INSERT INTO 
			report_key (key, owner, executable, reported_time, reported_by)
			 VALUES (?1, ?2, ?3, ?4, ?5)`
		).bind(key, info?.owner.toBase58(), Number(info?.executable), getCurrentTime(), userKey);
		await stmt.run();

		return c.json('Success');
	} catch (err) {
		console.log(err);
		if (err instanceof ZodError) {
			return c.json('Bad request', 400);
		}
		return c.json('Error', 500);
	}
});

export default app;
