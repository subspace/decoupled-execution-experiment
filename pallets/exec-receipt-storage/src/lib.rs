#![cfg_attr(not(feature = "std"), no_std)]

//! Exec Receipt Storage

use std::hash::Hash;

use sp_std::prelude::*;

use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{ChangeMembers, EnsureOrigin, Get, InitializeMembers},
};
use frame_system::ensure_signed;
use sp_runtime::RuntimeDebug;
use sp_std::fmt::Debug;

// #[cfg(test)]
// mod tests;

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type GenericSignature: Into<sp_core::sr25519::Signature>
        + Encode
        + Decode
        + Default
        + Debug
        + Clone
        + PartialEq
        + Eq;
}

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, RuntimeDebug)]
pub struct Receipt<Hash, AccountId, Signature> {
    final_root_balance: Hash,
    last_block: Hash, //current
    executor: AccountId,
    signed_root_balance: Signature,
}

decl_storage! {
    trait Store for Module<T: Config> as ReceiptStore {
        Received get(fn received): Vec<Receipt<<T as frame_system::Config>::Hash, <T as frame_system::Config>::AccountId, T::GenericSignature>>;
        Validated get(fn validated): map hasher(blake2_128_concat) u32 => Receipt<<T as frame_system::Config>::Hash, <T as frame_system::Config>::AccountId, T::GenericSignature>;
    }
}

decl_event! (
    pub enum Event<T>
    where
        <T as frame_system::Config>::Hash,
    {
        // Received receipt from executor
        Received(Hash),
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // method gossip
        #[weight = 10_000]
        pub fn gossip_receipt(origin, receipt: Receipt<<T as frame_system::Config>::Hash, <T as frame_system::Config>::AccountId, T::GenericSignature>) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let mut received = Received::<T>::get();
            let hash = receipt.last_block;
            received.push(receipt);
            Received::<T>::put(received);
            Self::deposit_event(RawEvent::Received(hash));
            Ok(())
        }
    }
}
