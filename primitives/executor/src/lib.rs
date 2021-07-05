// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Primitives for EXECUTOR integration, suitable for WASM compilation.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

// use std::collections::HashSet;
// use std::sync::Arc;

// use exec_membership_runtime::ExecutorMemberApi;
#[cfg(feature = "std")]
use serde::Serialize;

use codec::{Encode, Decode, Input, Codec};
// use sp_api::{ApiErrorFor, BlockId, BlockT, ProvideRuntimeApi};
// use sp_blockchain::HeaderBackend;
// use sp_keystore::CryptoStore;
use sp_runtime::{ConsensusEngineId, RuntimeDebug, traits::NumberFor};
use sp_std::borrow::Cow;
use sp_std::vec::Vec;
#[cfg(feature = "std")]
use sp_keystore::{SyncCryptoStorePtr, SyncCryptoStore};

#[cfg(feature = "std")]
use log::debug;

/// Key type for EXECUTOR module.
pub const KEY_TYPE: sp_core::crypto::KeyTypeId = sp_core::crypto::KeyTypeId(*b"exec");

mod app {
	use sp_application_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, crate::KEY_TYPE);
}

sp_application_crypto::with_pair! {
	/// The EXECUTOR crypto scheme defined via the keypair type.
	pub type AuthorityPair = app::Pair;
}

/// Identity of a EXECUTOR authority.
pub type AuthorityId = app::Public;

/// Signature for a EXECUTOR authority.
pub type AuthoritySignature = app::Signature;
