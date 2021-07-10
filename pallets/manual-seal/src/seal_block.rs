// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Block sealing utilities

use crate::{Error, rpc, CreatedBlock, ConsensusDataProvider};
use std::sync::Arc;
use codec::Encode;
use exec_membership_runtime::ExecutorMemberApi;
use exec_receipt_storage_runtime::ReceiptBuilderApi;
use executor_discovery::am_i_executor;
use sp_core::Public;
use sp_keystore::{SyncCryptoStore, SyncCryptoStorePtr};
use sp_runtime::{generic::{BlockId}, traits::{Block as BlockT, Header as HeaderT}};
use futures::prelude::*;
use sc_transaction_pool::txpool::{self, ExtrinsicFor};
use sp_consensus::{
	self, BlockImport, Environment, Proposer, ForkChoiceStrategy,
	BlockImportParams, BlockOrigin, ImportResult, SelectChain,
};
use sp_blockchain::HeaderBackend;
use std::collections::HashMap;
use std::time::Duration;
use sp_inherents::InherentDataProviders;
use sp_api::{ApiErrorFor, ProvideRuntimeApi, TransactionFor};
use sp_executor::{AuthorityId, KEY_TYPE, Receipt, AuthoritySignature};
use std::collections::HashSet;
use codec::Decode;
use std::convert::TryFrom;

/// max duration for creating a proposal in secs
pub const MAX_PROPOSAL_DURATION: u64 = 10;

/// params for sealing a new block
pub struct SealBlockParams<'a, B: BlockT, BI, SC, C: ProvideRuntimeApi<B>, E, P: txpool::ChainApi> {
	/// if true, empty blocks(without extrinsics) will be created.
	/// otherwise, will return Error::EmptyTransactionPool.
	pub create_empty: bool,
	/// instantly finalize this block?
	pub finalize: bool,
	/// specify the parent hash of the about-to-created block
	pub parent_hash: Option<<B as BlockT>::Hash>,
	/// sender to report errors/success to the rpc.
	pub sender: rpc::Sender<CreatedBlock<<B as BlockT>::Hash>>,
	/// transaction pool
	pub pool: Arc<txpool::Pool<P>>,
	/// header backend
	pub client: Arc<C>,
	/// Environment trait object for creating a proposer
	pub env: &'a mut E,
	/// SelectChain object
	pub select_chain: &'a SC,
	/// Digest provider for inclusion in blocks.
	pub consensus_data_provider: Option<&'a dyn ConsensusDataProvider<B, Transaction = TransactionFor<C, B>>>,
	/// block import object
	pub block_import: &'a mut BI,
	/// inherent data provider
	pub inherent_data_provider: &'a InherentDataProviders,

	// pub keystore: SyncCryptoStorePtr,	
}

/// seals a new block with the given params
pub async fn seal_block<B, BI, SC, C, E, P>(
	SealBlockParams {
		create_empty,
		finalize,
		pool,
		parent_hash,
		client,
		select_chain,
		block_import,
		env,
		inherent_data_provider,
		consensus_data_provider: digest_provider,
		mut sender,
		..
	}: SealBlockParams<'_, B, BI, SC, C, E, P>,
	keystore: SyncCryptoStorePtr,
)
	where
		B: BlockT + Unpin + 'static,
		BI: BlockImport<B, Error = sp_consensus::Error, Transaction = sp_api::TransactionFor<C, B>>
			+ Send + Sync + 'static,
		C: ProvideRuntimeApi<B> + 'static + HeaderBackend<B>,
		<C as ProvideRuntimeApi<B>>::Api: ExecutorMemberApi<B, AuthorityId> + ReceiptBuilderApi<B, <B as BlockT>::Hash, sp_executor::AuthorityId, sp_executor::AuthoritySignature>,
		E: Environment<B>,
		E::Proposer: Proposer<B, Transaction = TransactionFor<C, B>>,
		P: txpool::ChainApi<Block=B>,
		SC: SelectChain<B>,
		TransactionFor<C, B>: 'static,
{
	let future = async {
		if pool.validated_pool().status().ready == 0 && !create_empty {
			return Err(Error::EmptyTransactionPool)
		}

		// get the header to build this new block on.
		// use the parent_hash supplied via `EngineCommand`
		// or fetch the best_block.
		let parent = match parent_hash {
			Some(hash) => {
				match client.header(BlockId::Hash(hash))? {
					Some(header) => header,
					None => return Err(Error::BlockNotFound(format!("{}", hash))),
				}
			}
			None => select_chain.best_chain()?
		};

		let proposer = env.init(&parent)
			.map_err(|err| Error::StringError(format!("{:?}", err))).await?;
		let id = inherent_data_provider.create_inherent_data()?;
		let inherents_len = id.len();

		let digest = if let Some(digest_provider) = digest_provider {
			digest_provider.create_digest(&parent, &id)?
		} else {
			Default::default()
		};

		let proposal = proposer.propose(id.clone(), digest, Duration::from_secs(MAX_PROPOSAL_DURATION), false.into())
			.map_err(|err| Error::StringError(format!("{:?}", err))).await?;

		if proposal.block.extrinsics().len() == inherents_len && !create_empty {
			return Err(Error::EmptyTransactionPool)
		}

		let (header, body) = proposal.block.deconstruct();
		let mut params = BlockImportParams::new(BlockOrigin::Own, header.clone());
		params.body = Some(body);
		params.finalized = finalize;
		params.fork_choice = Some(ForkChoiceStrategy::LongestChain);
		params.storage_changes = Some(proposal.storage_changes);

		if let Some(digest_provider) = digest_provider {
			digest_provider.append_block_import(&parent, &mut params, &id)?;
		}

		match block_import.import_block(params, HashMap::new())? {
			ImportResult::Imported(aux) => {

				let mut executor: Option<AuthorityId> = None;
				let local_pub_keys = 
				SyncCryptoStore::sr25519_public_keys(&*keystore, KEY_TYPE)				
				.into_iter()
				.collect::<HashSet<_>>();
				let id = BlockId::hash(client.info().best_hash);
				for key in local_pub_keys.iter() {
					let authority = AuthorityId::from(*key);
					if client.runtime_api().is_executor(&id, authority.clone()).expect("should not fail") {
						executor = Some(authority);
						break;
					}
				}				

				if let Some(authority) = executor { 
					let state_root = header.state_root();
					let best_hash = client.info().best_hash;
					let mut encoded = Vec::new();
					state_root.encode_to(&mut encoded);
					let signature = SyncCryptoStore::sign_with(&*keystore,
						KEY_TYPE,
						&authority.to_public_crypto_pair(),
						&encoded[..],
					).expect("should not fail");
			
					let er = Receipt {
						final_root_balance: state_root.clone(),
						last_block: best_hash.clone(), //current
						executor: authority.clone(),
						signed_root_balance: AuthoritySignature::try_from(signature).expect("should not fail"),
					};
			
					let encoded_xt = client.runtime_api().build_extrinsic(&id,er).expect("don't fail");
					// let xt = ExtrinsicFor::decode(&mut &*encoded_xt).expect("don't fail");
			
					// let _r = pool
					// .submit_one(&id, sp_transaction_pool::TransactionSource::External, xt).await.expect("don't fail");
				
				}

				Ok(CreatedBlock { hash: <B as BlockT>::Header::hash(&header), aux })
			},
			other => Err(other.into()),
		}
	};

	rpc::send_result(&mut sender, future.await)
}