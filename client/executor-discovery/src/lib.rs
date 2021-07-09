use std::{collections::HashSet, sync::Arc};

use exec_membership_runtime::ExecutorMemberApi;
use sp_api::{ApiErrorFor, BlockId, BlockT, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_executor::{AuthorityId, KEY_TYPE};
use sp_keystore::{SyncCryptoStore, SyncCryptoStorePtr};

pub async fn am_i_executor<Client, Block>(
    key_store: SyncCryptoStorePtr,
    client: &Client,
) -> Result<Option<AuthorityId>, ApiErrorFor<Client, Block>>
where
    Block: BlockT + Unpin + 'static,
    Client: ProvideRuntimeApi<Block> + 'static + HeaderBackend<Block>,
    <Client as ProvideRuntimeApi<Block>>::Api: ExecutorMemberApi<Block, AuthorityId>,
{
    let local_pub_keys = SyncCryptoStore::sr25519_public_keys(&*key_store,
        KEY_TYPE)
        .into_iter()
        .collect::<HashSet<_>>();
    let id = BlockId::hash(client.info().best_hash);
    for key in local_pub_keys.iter() {
        let authority = AuthorityId::from(*key);
        let executor = client.runtime_api().is_executor(&id, authority.clone())?;
        if executor {
            return Ok(Some(authority));
        }
    }
    Ok(None)
}
