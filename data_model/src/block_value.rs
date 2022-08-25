//! This module contains [`BlockValue`] and [`BlockHeaderValue`] structures, their implementation and related traits and
//! instructions implementations.
#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
use core::cmp::Ordering;

#[cfg(not(target_arch = "aarch64"))]
use derive_more::Into;

use derive_more::{AsRef, Deref, Display, From};
use iroha_crypto::{Hash, HashOf, MerkleTree};
use iroha_schema::IntoSchema;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{
    events::Event,
    transaction::{VersionedRejectedTransaction, VersionedTransaction, VersionedValidTransaction},
};

/// Block header
#[derive(
    Debug, Clone, Display, PartialEq, Eq, Decode, Encode, Deserialize, Serialize, IntoSchema,
)]
#[display(fmt = "Block №{height} (hash: {transactions_hash});")]
pub struct BlockHeaderValue {
    /// Unix time (in milliseconds) of block forming by a peer.
    pub timestamp: u128,
    /// a number of blocks in the chain up to the block.
    pub height: u64,
    /// Hash of a previous block in the chain.
    /// Is an array of zeros for the first block.
    pub previous_block_hash: Hash,
    /// Hash of merkle tree root of the tree of valid transactions' hashes.
    pub transactions_hash: HashOf<MerkleTree<VersionedTransaction>>,
    /// Hash of merkle tree root of the tree of rejected transactions' hashes.
    pub rejected_transactions_hash: HashOf<MerkleTree<VersionedTransaction>>,
    /// Hashes of the blocks that were rejected by consensus.
    pub invalidated_blocks_hashes: Vec<Hash>,
    /// Hash of the most recent block
    pub current_block_hash: Hash,
}

impl PartialOrd for BlockHeaderValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockHeaderValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

/// Representation of block on blockchain
#[derive(
    Debug, Display, Clone, PartialEq, Eq, Decode, Encode, Serialize, Deserialize, IntoSchema,
)]
#[display(fmt = "({})", header)]
pub struct BlockValue {
    /// Header
    pub header: BlockHeaderValue,
    /// Array of transactions
    pub transactions: Vec<VersionedValidTransaction>,
    /// Array of rejected transactions.
    pub rejected_transactions: Vec<VersionedRejectedTransaction>,
    /// Event recommendations
    pub event_recommendations: Vec<Event>,
}

impl PartialOrd for BlockValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.header.cmp(&other.header)
    }
}

impl From<BlockValue> for crate::value::Value {
    fn from(block_value: BlockValue) -> Self {
        crate::value::Value::Block(block_value.into())
    }
}

#[cfg(target_arch = "aarch64")]
impl From<BlockValueWrapper> for BlockValue {
    fn from(block_value: BlockValueWrapper) -> Self {
        *block_value.0
    }
}

impl IntoSchema for BlockValueWrapper {
    fn type_name() -> String {
        BlockValue::type_name()
    }

    fn schema(map: &mut iroha_schema::MetaMap) {
        BlockValue::schema(map);
    }
}

/// Cross-platform wrapper for `BlockValue`.
#[cfg(target_arch = "aarch64")]
#[derive(
    AsRef,
    Clone,
    Debug,
    Decode,
    Deref,
    Deserialize,
    Encode,
    Eq,
    From,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[as_ref(forward)]
#[deref(forward)]
#[from(forward)]
#[serde(transparent)]
pub struct BlockValueWrapper(Box<BlockValue>);

/// Cross-platform wrapper for `BlockValue`.
#[cfg(not(target_arch = "aarch64"))]
#[derive(
    AsRef,
    Clone,
    Debug,
    Decode,
    Deref,
    Deserialize,
    Encode,
    Eq,
    From,
    Into,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(transparent)]
pub struct BlockValueWrapper(BlockValue);

/// The prelude re-exports most commonly used traits, structs and macros from this crate.
pub mod prelude {
    pub use super::{BlockHeaderValue, BlockValue, BlockValueWrapper};
}
