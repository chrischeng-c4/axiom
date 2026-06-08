//! Pluggable storage backend trait.
//!
//! This module abstracts the read/write surface that the field indexes
//! in [`crate::storage`] currently perform directly against in-memory
//! `BTreeMap`s. The trait is the seam between the existing in-memory
//! `Engine` and the log-structured backend in [`crate::storage_lsm`].
//!
//! ### Design notes
//!
//! * Operations are intentionally narrow — they mirror the primitive
//!   things the in-memory engine does on a posting list (insert,
//!   delete, look up by exact key, scan a key range). Higher-level
//!   query evaluation (BM25, AND/OR, range fusion) still lives in
//!   `storage.rs`.
//! * The trait is `Send + Sync`. The LSM implementation is internally
//!   synchronised; callers do not need to wrap the backend in a lock.
//! * Operations are **synchronous** in this v1. The LSM backend runs
//!   compaction on a background thread but the user-facing surface is
//!   blocking. We chose sync over async to keep the trait callable
//!   from inside the `Engine`'s `RwLock` critical sections without
//!   pulling tokio into every code path. The LSM internally spawns a
//!   compaction thread; that's the only background concurrency.
//! * "Partition" is the `hash(field) % 4` shard local to the pod
//!   (README §2). Callers compute this and pass it in.
//! * Values are opaque bytes from the backend's perspective. The
//!   in-memory engine encodes its own posting payload (term frequency
//!   for text, set membership marker for set/keyword, etc.). The LSM
//!   does not introspect them.

use std::sync::Arc;

use anyhow::Result;

/// A logical posting key: the (collection, partition, field-key)
/// triple addressed by a backend lookup.
///
/// The `key` portion is the field-specific term — token for text,
/// keyword/element string for keyword/set, big-endian
/// [`crate::storage::SortableF64`] bytes for number. Backends treat
/// it as an opaque byte string with the usual lexicographic ordering.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PostingKey {
    pub collection: String,
    pub partition: u8,
    pub key: Vec<u8>,
}

impl PostingKey {
    pub fn new(collection: impl Into<String>, partition: u8, key: impl Into<Vec<u8>>) -> Self {
        Self {
            collection: collection.into(),
            partition,
            key: key.into(),
        }
    }
}

/// One entry inside a posting list.
///
/// `external_id` is the document handle the caller addresses lumen
/// with. `payload` is the per-(field, eid) opaque blob written by the
/// caller — for example, `u32::to_le_bytes` of a term frequency.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostingEntry {
    pub external_id: String,
    pub payload: Vec<u8>,
}

/// One row of a [`Backend::recover`] dump — every live posting that
/// survived WAL replay + SST merge, addressed by its full
/// `(collection, partition, key, eid)` tuple plus the opaque payload
/// the caller originally wrote.
///
/// Tombstoned entries are filtered out by the backend; recovery sees
/// the same materialised view that a fresh `posting()` call would
/// have surfaced for each key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecoveredPosting {
    pub collection: String,
    pub partition: u8,
    pub key: Vec<u8>,
    pub external_id: String,
    pub payload: Vec<u8>,
}

/// Pluggable storage trait — see module docs for context.
///
/// **All operations must be `Send + Sync` and idempotent on retry**
/// (the LSM impl is naturally idempotent because of the WAL replay
/// path).
pub trait Backend: Send + Sync {
    /// Insert (or upsert) `(eid, payload)` into the posting list at
    /// `(collection, partition, key)`. If `eid` already exists, its
    /// payload is replaced.
    fn put_posting(
        &self,
        collection: &str,
        partition: u8,
        key: &[u8],
        eid: &str,
        payload: &[u8],
    ) -> Result<()>;

    /// Remove the entry for `eid` from `(collection, partition, key)`.
    /// No-op if not present. Writes a tombstone record to the WAL so
    /// recovery sees the delete.
    fn delete_posting(&self, collection: &str, partition: u8, key: &[u8], eid: &str) -> Result<()>;

    /// Return the full posting list at `(collection, partition, key)`
    /// in `external_id` order. The result is materialised because the
    /// LSM has to merge memtable + N SSTs; a streaming iterator would
    /// hand back references across mmap'd regions that we'd rather not
    /// expose this far.
    fn posting(&self, collection: &str, partition: u8, key: &[u8]) -> Result<Vec<PostingEntry>>;

    /// Range scan: every key `k` with `lo <= k < hi`, paired with its
    /// posting list. `None` bounds are open. Used by `RangeQuery` on
    /// number fields.
    fn scan_range(
        &self,
        collection: &str,
        partition: u8,
        lo: Option<&[u8]>,
        hi: Option<&[u8]>,
    ) -> Result<Vec<(Vec<u8>, Vec<PostingEntry>)>>;

    /// Force the active memtable to a fresh SST. Blocks until the SST
    /// is on disk and the WAL has been retired. No-op if the memtable
    /// is empty.
    fn flush(&self) -> Result<()>;

    /// Wait for any in-flight background compaction to drain. Used by
    /// tests; production callers should not need to call this.
    fn compact(&self) -> Result<()>;

    /// Cold-start recovery: walk every live posting (memtable + SSTs,
    /// tombstones already applied) and return one [`RecoveredPosting`]
    /// per `(collection, partition, key, eid)` tuple.
    ///
    /// The result is materialised in full because the caller
    /// ([`crate::storage::Engine::new_with_backend`]) needs to rebuild
    /// schemas first and then re-hydrate per-collection in-memory
    /// caches. For collections with millions of postings this is a
    /// one-shot scan at boot; future versions can defer-load via a
    /// metadata SSTable, but v1 keeps the simpler contract.
    fn recover(&self) -> Result<Vec<RecoveredPosting>>;
}

/// Convenience alias — most callers will hand the backend around
/// behind an `Arc` so the `Engine` and any background flush hooks
/// can both hold a handle.
pub type SharedBackend = Arc<dyn Backend>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posting_key_constructor_normalizes_to_bytes() {
        let k = PostingKey::new("users", 3, b"alice".to_vec());
        assert_eq!(k.collection, "users");
        assert_eq!(k.partition, 3);
        assert_eq!(k.key, b"alice");
    }

    #[test]
    fn posting_key_accepts_str_and_string_alike() {
        let a = PostingKey::new("c", 0, "hello");
        let b = PostingKey::new(String::from("c"), 0, String::from("hello"));
        assert_eq!(a, b);
    }

    #[test]
    fn posting_key_lex_ordering() {
        // BTreeSet / scan_range rely on lexicographic key order.
        let mut keys: Vec<PostingKey> = vec![
            PostingKey::new("u", 0, b"b".to_vec()),
            PostingKey::new("u", 0, b"a".to_vec()),
            PostingKey::new("u", 0, b"aa".to_vec()),
        ];
        keys.sort();
        assert_eq!(keys[0].key, b"a");
        assert_eq!(keys[1].key, b"aa");
        assert_eq!(keys[2].key, b"b");
    }

    #[test]
    fn posting_entry_ord_by_external_id() {
        let mut entries = vec![
            PostingEntry {
                external_id: "z".into(),
                payload: vec![],
            },
            PostingEntry {
                external_id: "a".into(),
                payload: vec![],
            },
            PostingEntry {
                external_id: "m".into(),
                payload: vec![],
            },
        ];
        entries.sort();
        assert_eq!(entries[0].external_id, "a");
        assert_eq!(entries[1].external_id, "m");
        assert_eq!(entries[2].external_id, "z");
    }

    #[test]
    fn recovered_posting_round_trip() {
        let r = RecoveredPosting {
            collection: "c".into(),
            partition: 1,
            key: b"k".to_vec(),
            external_id: "u1".into(),
            payload: vec![0xAA, 0xBB],
        };
        let r2 = r.clone();
        assert_eq!(r, r2);
        assert!(format!("{r:?}").contains("RecoveredPosting"));
    }
}
