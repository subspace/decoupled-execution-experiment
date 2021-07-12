//! Trait for building receipt extrinsic.
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

use frame_support::{RuntimeDebug, debug};
use sp_runtime::OpaqueExtrinsic;
use sp_executor::Receipt;

#[cfg(not(feature = "std"))]
use sp_std::prelude::Vec;

use codec::Codec;

sp_api::decl_runtime_apis! {
	pub trait ReceiptBuilderApi<Hash, AuthorityId, Signature> where
	Hash: Codec,
	AuthorityId: Codec,
	Signature: Codec,
	{    
        fn build_extrinsic(er: Receipt<Hash, AuthorityId, Signature>) -> Vec<u8>;
    }
}
