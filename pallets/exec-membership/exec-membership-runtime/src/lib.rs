//! A Simple executor membership trait which holds executor authorities.
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

use frame_support::{RuntimeDebug, debug};
use sp_runtime::traits::{self, Saturating, One};
use sp_std::fmt;
#[cfg(not(feature = "std"))]
use sp_std::prelude::Vec;

use codec::Codec;

sp_api::decl_runtime_apis! {
	pub trait ExecutorMemberApi<AccountId> where
		AccountId: Codec,
	{    
        fn is_executor(account: AccountId) -> bool;
    }
}