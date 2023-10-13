from typing import List
import borsh_construct as borsh
from solana.transaction import TransactionInstruction, AccountMeta
from solana.publickey import PublicKey
class VoteInstruction:
	schema = borsh.CStruct(
		"tag" / borsh.U8,
		"user_key" / borsh.U8[32],
		"vote_value" / borsh.U8,
	)
	def serialize(self,
		user_key: List[int],
		vote_value: int,
	) -> str:
		return self.schema.build({
			"tag": 0,
			"user_key": user_key,
			"vote_value": vote_value,
		})
	def getInstruction(self,
		user_key: List[int],
		vote_value: int,
programId: PublicKey,
system_program: PublicKey,
voter: PublicKey,
reputation_state_account: PublicKey,
user_vote_state_account: PublicKey,
voter_stake_accounts: List[PublicKey],
) -> TransactionInstruction:
		data = self.serialize(
		user_key,
		vote_value,
)
		keys: List[AccountMeta] = []
		keys.append(AccountMeta(system_program,
			False, False))
		keys.append(AccountMeta(voter,
			True, True))
		keys.append(AccountMeta(reputation_state_account,
			False, True))
		keys.append(AccountMeta(user_vote_state_account,
			False, True))
		for k in voter_stake_accounts:
			keys.append(AccountMeta(k,
			False, False))
		return TransactionInstruction(keys, programId, data)
