//! Shared immutable-segment and archive coordination contracts.

use std::{collections::BTreeSet, sync::Arc};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use storage_object::{ObjectStore, ObjectStoreError, ObjectVersion, PutCondition};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SegmentError {
    #[error("segment codec failed: {message}")]
    Codec { message: String },
    #[error("segment partition is invalid: {partition}")]
    InvalidPartition { partition: String },
    #[error("archive object key is repeated: {key}")]
    DuplicateObject { key: String },
    #[error("archive manifest key was already used by an immutable object: {key}")]
    ManifestKeyCollision { key: String },
    #[error("archive transaction cannot commit after an earlier upload failed")]
    TransactionFailed,
    #[error("existing immutable object {key} has different content")]
    ImmutableObjectChanged { key: String },
    #[error(transparent)]
    ObjectStore(#[from] ObjectStoreError),
}

pub type Result<T> = std::result::Result<T, SegmentError>;

/// Product codec for records stored in one immutable segment.
pub trait RecordCodec<Record>: Send + Sync {
    fn encode(&self, records: &[Record]) -> Result<Vec<u8>>;
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Record>>;
}

/// Product policy that selects a stable partition for one record.
pub trait Partitioner<Record>: Send + Sync {
    fn partition(&self, record: &Record) -> Result<String>;
}

/// Durable local segment boundary. A product supplies its descriptor type.
pub trait SegmentStore<Record>: Send + Sync {
    type Descriptor: Clone + Send + Sync;

    fn write_immutable(&self, partition: &str, records: &[Record]) -> Result<Self::Descriptor>;
    fn read(&self, descriptor: &Self::Descriptor) -> Result<Vec<Record>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveObject {
    pub key: String,
    pub bytes: Vec<u8>,
    pub content_type: String,
}

impl ArchiveObject {
    pub fn new(key: impl Into<String>, bytes: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            bytes,
            content_type: content_type.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArchivedObject {
    pub key: String,
    pub size: u64,
    pub content_type: String,
    pub sha256: String,
    pub version: ObjectVersion,
}

/// Receipt returned only after the final manifest write succeeds.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArchiveCommit {
    pub objects: Vec<ArchivedObject>,
    pub manifest: ArchivedObject,
}

#[derive(Clone)]
pub struct ArchiveCoordinator {
    store: Arc<dyn ObjectStore>,
}

impl ArchiveCoordinator {
    pub fn new(store: Arc<dyn ObjectStore>) -> Self {
        Self { store }
    }

    pub fn begin(&self) -> ArchiveTransaction {
        ArchiveTransaction {
            store: self.store.clone(),
            objects: Vec::new(),
            keys: BTreeSet::new(),
            failed: false,
        }
    }
}

pub struct ArchiveTransaction {
    store: Arc<dyn ObjectStore>,
    objects: Vec<ArchivedObject>,
    keys: BTreeSet<String>,
    failed: bool,
}

impl ArchiveTransaction {
    /// Write one immutable object. A retry accepts an existing byte-identical
    /// object, but it rejects content changes under the same key.
    pub fn put(&mut self, object: ArchiveObject) -> Result<ArchivedObject> {
        if self.failed {
            return Err(SegmentError::TransactionFailed);
        }
        if !self.keys.insert(object.key.clone()) {
            return Err(SegmentError::DuplicateObject { key: object.key });
        }
        match put_immutable(self.store.as_ref(), object) {
            Ok(receipt) => {
                self.objects.push(receipt.clone());
                Ok(receipt)
            }
            Err(error) => {
                self.failed = true;
                Err(error)
            }
        }
    }

    /// Write the manifest last. No `ArchiveCommit` exists before this call.
    pub fn commit(mut self, manifest: ArchiveObject) -> Result<ArchiveCommit> {
        if self.failed {
            return Err(SegmentError::TransactionFailed);
        }
        if self.keys.contains(&manifest.key) {
            return Err(SegmentError::ManifestKeyCollision { key: manifest.key });
        }
        let manifest = match put_immutable(self.store.as_ref(), manifest) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.failed = true;
                return Err(error);
            }
        };
        Ok(ArchiveCommit {
            objects: self.objects,
            manifest,
        })
    }
}

fn put_immutable(store: &dyn ObjectStore, object: ArchiveObject) -> Result<ArchivedObject> {
    let sha256 = hex_sha256(&object.bytes);
    let meta = match store.put(
        &object.key,
        &object.bytes,
        &object.content_type,
        PutCondition::IfAbsent,
    ) {
        Ok(meta) => meta,
        Err(ObjectStoreError::PreconditionFailed { .. }) => {
            let existing = store.get(&object.key)?;
            if existing.bytes != object.bytes || existing.meta.content_type != object.content_type {
                return Err(SegmentError::ImmutableObjectChanged { key: object.key });
            }
            existing.meta
        }
        Err(error) => return Err(error.into()),
    };
    Ok(ArchivedObject {
        key: object.key,
        size: object.bytes.len() as u64,
        content_type: object.content_type,
        sha256,
        version: meta.version,
    })
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut text, "{byte:02x}").expect("write to String cannot fail");
    }
    text
}
