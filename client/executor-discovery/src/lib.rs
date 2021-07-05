use std::{collections::HashSet, sync::Arc};

use exec_membership_runtime::ExecutorMemberApi;
use sp_api::{ApiErrorFor, BlockId, BlockT, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_executor::{AuthorityId, KEY_TYPE};
use sp_keystore::CryptoStore;

pub async fn am_i_executor<Client, Block>(
	key_store: Arc<dyn CryptoStore>,
	client: &Client,
)-> Result<bool, ApiErrorFor<Client, Block>> 
where
Block: BlockT + Unpin + 'static,
Client: ProvideRuntimeApi<Block> + Send + Sync + 'static + HeaderBackend<Block>,
<Client as ProvideRuntimeApi<Block>>::Api:
ExecutorMemberApi<Block, AuthorityId>,	
{
	let local_pub_keys = key_store
		.sr25519_public_keys(KEY_TYPE)
		.await
		.into_iter()
		.collect::<HashSet<_>>();
	let id = BlockId::hash(client.info().best_hash);
	for key in local_pub_keys.iter() {
		let executor = client.runtime_api()
		.is_executor(&id, AuthorityId::from(*key))?;
		if executor { return Ok(true) }
	}
	Ok(false)
}
