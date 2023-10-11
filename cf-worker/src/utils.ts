import { z } from 'zod';
import { PublicKey } from '@solana/web3.js';
import { sign } from 'tweetnacl';

export const TTL = 2 * 60;

export const isPubkey = (x: string) => {
	try {
		new PublicKey(x);
		return true;
	} catch {
		return false;
	}
};

export const pubkey = z.custom<string>((val) => isPubkey(String(val)));
export type pubkey = z.infer<typeof pubkey>;

export const generateMessage = (wallet: string): string => {
	const rnd = crypto.randomUUID();
	const msg = `Authenticate wallet (${wallet}) by signing the below:\n${rnd}`;
	return msg;
};

export const verifyMessage = (message: string, signature: string, wallet: string): boolean => {
	return sign.detached.verify(
		new TextEncoder().encode(message),
		new Uint8Array(Buffer.from(signature, 'hex')),
		new PublicKey(wallet).toBytes()
	);
};

export const getCurrentTime = (): number => {
	const now = Math.floor(new Date().getTime() / 1_000);
	return now;
};
