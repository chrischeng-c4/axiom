//! Shared immutable-segment and archive coordination contracts.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

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
    #[error("catalog key is invalid: {key}")]
    InvalidCatalogKey { key: String },
    #[error("catalog contains a duplicate key: {key}")]
    DuplicateCatalogKey { key: String },
    #[error("streaming catalog keys are not strictly increasing: {key}")]
    UnsortedCatalogKey { key: String },
    #[error("catalog page exceeds {limit} bytes")]
    CatalogPageTooLarge { limit: usize },
    #[error("catalog data is corrupt: {message}")]
    CorruptCatalog { message: String },
    #[error("catalog serialization failed: {message}")]
    Serialization { message: String },
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

pub const DEFAULT_CATALOG_PAGE_BYTES: usize = 64 * 1024;
/// Maximum page keys retained by the compatibility abort helper.
///
/// Production builds with larger catalogs must use `build_sorted_observed`
/// and persist page keys outside process memory.
pub const MAX_ABORT_TRACKED_CATALOG_PAGES: usize = 1_024;
const CATALOG_FORMAT_VERSION: u16 = 1;
const CATALOG_PAGE_FORMAT_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogEntry {
    pub key: String,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogPageRef {
    pub key: String,
    pub sha256: String,
    pub bytes: u64,
    pub entry_count: u64,
    pub first_key: String,
    pub last_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogRoot {
    pub format_version: u16,
    pub height: u16,
    pub entry_count: u64,
    pub page_bytes_limit: u32,
    pub root: CatalogPageRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogMutation {
    pub root: CatalogRoot,
    pub written_page_keys: Vec<String>,
    pub obsolete_page_keys: Vec<String>,
}

/// Result of a bounded-memory bulk build over an already sorted input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingCatalogBuild {
    pub root: CatalogRoot,
    pub written_page_count: u64,
    pub peak_buffer_bytes: usize,
}

/// A failed streaming build plus every page key it may have created.
///
/// The caller can clean these keys when the catalog prefix is private to the
/// failed transaction. No root was committed, so the pages are not a catalog.
#[derive(Debug)]
pub struct StreamingCatalogAbort {
    pub error: SegmentError,
    pub written_page_keys: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CatalogPage {
    format_version: u16,
    #[serde(flatten)]
    body: CatalogPageBody,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CatalogPageBody {
    Leaf { entries: Vec<CatalogEntry> },
    Branch { children: Vec<CatalogPageRef> },
}

#[derive(Clone)]
pub struct PagedCatalog {
    store: Arc<dyn ObjectStore>,
    prefix: String,
    page_bytes_limit: usize,
}

impl PagedCatalog {
    pub fn new(store: Arc<dyn ObjectStore>, prefix: impl Into<String>) -> Result<Self> {
        Self::with_page_bytes(store, prefix, DEFAULT_CATALOG_PAGE_BYTES)
    }

    pub fn with_page_bytes(
        store: Arc<dyn ObjectStore>,
        prefix: impl Into<String>,
        page_bytes_limit: usize,
    ) -> Result<Self> {
        let prefix = prefix.into().trim_matches('/').to_string();
        if prefix.is_empty()
            || prefix
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
        {
            return Err(SegmentError::InvalidCatalogKey { key: prefix });
        }
        if !(4 * 1024..=DEFAULT_CATALOG_PAGE_BYTES).contains(&page_bytes_limit) {
            return Err(SegmentError::CatalogPageTooLarge {
                limit: page_bytes_limit,
            });
        }
        Ok(Self {
            store,
            prefix,
            page_bytes_limit,
        })
    }

    pub fn build(
        &self,
        entries: impl IntoIterator<Item = CatalogEntry>,
    ) -> Result<CatalogMutation> {
        let mut sorted = BTreeMap::new();
        for entry in entries {
            self.validate_entry_key(&entry.key)?;
            let key = entry.key.clone();
            if sorted.insert(key.clone(), entry).is_some() {
                return Err(SegmentError::DuplicateCatalogKey { key });
            }
        }
        let entry_count = sorted.len() as u64;
        let leaves = self.pack_leaves(sorted.into_values().collect())?;
        let mut written = Vec::new();
        let mut level = leaves
            .into_iter()
            .map(|entries| {
                self.store_page(CatalogPageBody::Leaf { entries })
                    .inspect(|reference| {
                        written.push(reference.key.clone());
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut height = 0_u16;
        while level.len() > 1 {
            height = height
                .checked_add(1)
                .ok_or_else(|| SegmentError::CorruptCatalog {
                    message: "catalog height exhausted u16".to_string(),
                })?;
            let groups = self.pack_children(level)?;
            level = groups
                .into_iter()
                .map(|children| {
                    self.store_page(CatalogPageBody::Branch { children })
                        .inspect(|reference| {
                            written.push(reference.key.clone());
                        })
                })
                .collect::<Result<Vec<_>>>()?;
        }
        let root = CatalogRoot {
            format_version: CATALOG_FORMAT_VERSION,
            height,
            entry_count,
            page_bytes_limit: self.page_bytes_limit as u32,
            root: level.pop().expect("catalog build always creates one leaf"),
        };
        self.validate_root(&root)?;
        Ok(CatalogMutation {
            root,
            written_page_keys: written,
            obsolete_page_keys: Vec::new(),
        })
    }

    /// Build a catalog without retaining all entries or page references.
    ///
    /// The caller owns external sorting. This method rejects an equal or
    /// decreasing key. It retains at most one page-sized entry buffer and one
    /// page-sized reference buffer for each tree level.
    pub fn build_sorted(
        &self,
        entries: impl IntoIterator<Item = Result<CatalogEntry>>,
    ) -> Result<StreamingCatalogBuild> {
        self.build_sorted_observed(entries, |_| Ok(()))
    }

    /// Build a sorted catalog and retain page keys for abort cleanup.
    ///
    /// This compatibility API has a hard limit of
    /// `MAX_ABORT_TRACKED_CATALOG_PAGES`. It fails before retaining another
    /// key. Use `build_sorted_observed` for catalogs that can exceed the cap.
    pub fn build_sorted_with_abort(
        &self,
        entries: impl IntoIterator<Item = Result<CatalogEntry>>,
    ) -> std::result::Result<StreamingCatalogBuild, StreamingCatalogAbort> {
        let mut written_page_keys = Vec::new();
        self.build_sorted_observed(entries, |reference| {
            if written_page_keys.len() == MAX_ABORT_TRACKED_CATALOG_PAGES {
                return Err(SegmentError::CorruptCatalog {
                    message: format!(
                        "compatibility abort cleanup reached its bounded limit of {} pages; use build_sorted_observed",
                        MAX_ABORT_TRACKED_CATALOG_PAGES
                    ),
                });
            }
            written_page_keys.push(reference.key.clone());
            Ok(())
        })
        .map_err(|error| StreamingCatalogAbort {
            error,
            written_page_keys,
        })
    }

    /// Build a sorted catalog while reporting each durable page immediately.
    ///
    /// A production caller can persist the keys in a disk-backed ledger. This
    /// keeps abort cleanup bounded by the catalog page size instead of keeping
    /// one `String` per uploaded page in memory. If the observer rejects a
    /// page, this method deletes that just-written page before it returns.
    pub fn build_sorted_observed(
        &self,
        entries: impl IntoIterator<Item = Result<CatalogEntry>>,
        mut observe: impl FnMut(&CatalogPageRef) -> Result<()>,
    ) -> Result<StreamingCatalogBuild> {
        self.build_sorted_inner(entries, &mut observe)
    }

    fn build_sorted_inner<F>(
        &self,
        entries: impl IntoIterator<Item = Result<CatalogEntry>>,
        observe: &mut F,
    ) -> Result<StreamingCatalogBuild>
    where
        F: FnMut(&CatalogPageRef) -> Result<()>,
    {
        let mut leaf = Vec::<CatalogEntry>::new();
        let mut levels = Vec::<Vec<CatalogPageRef>>::new();
        let empty_leaf_bytes = self.page_size(&CatalogPageBody::Leaf {
            entries: Vec::new(),
        })?;
        let empty_branch_bytes = self.page_size(&CatalogPageBody::Branch {
            children: Vec::new(),
        })?;
        let mut leaf_bytes = empty_leaf_bytes;
        let mut level_bytes = Vec::<usize>::new();
        let mut last_key = None::<String>;
        let mut entry_count = 0_u64;
        let mut written_page_count = 0_u64;
        let mut peak_buffer_bytes = 0_usize;

        for entry in entries {
            let entry = entry?;
            self.validate_entry_key(&entry.key)?;
            if last_key
                .as_ref()
                .is_some_and(|previous| previous >= &entry.key)
            {
                return Err(SegmentError::UnsortedCatalogKey { key: entry.key });
            }
            last_key = Some(entry.key.clone());
            entry_count =
                entry_count
                    .checked_add(1)
                    .ok_or_else(|| SegmentError::CorruptCatalog {
                        message: "catalog entry count exhausted u64".to_string(),
                    })?;
            let encoded_entry =
                serde_json::to_vec(&entry).map_err(|error| SegmentError::Serialization {
                    message: error.to_string(),
                })?;
            let added = encoded_entry.len() + usize::from(!leaf.is_empty());
            if !leaf.is_empty() && leaf_bytes.saturating_add(added) > self.page_bytes_limit {
                let reference = self.store_page(CatalogPageBody::Leaf {
                    entries: std::mem::take(&mut leaf),
                })?;
                self.observe_streaming_page(observe, &reference)?;
                written_page_count = written_page_count.saturating_add(1);
                self.push_streaming_ref(
                    &mut levels,
                    &mut level_bytes,
                    empty_branch_bytes,
                    0,
                    reference,
                    &mut written_page_count,
                    observe,
                )?;
                leaf_bytes = empty_leaf_bytes;
            }
            let added = encoded_entry.len() + usize::from(!leaf.is_empty());
            if leaf_bytes.saturating_add(added) > self.page_bytes_limit {
                return Err(SegmentError::CatalogPageTooLarge {
                    limit: self.page_bytes_limit,
                });
            }
            leaf_bytes = leaf_bytes.saturating_add(added);
            leaf.push(entry);
            peak_buffer_bytes = peak_buffer_bytes
                .max(leaf_bytes.saturating_add(level_bytes.iter().copied().sum::<usize>()));
        }

        if entry_count == 0 {
            let root = self.store_page(CatalogPageBody::Leaf {
                entries: Vec::new(),
            })?;
            self.observe_streaming_page(observe, &root)?;
            return Ok(StreamingCatalogBuild {
                root: CatalogRoot {
                    format_version: CATALOG_FORMAT_VERSION,
                    height: 0,
                    entry_count: 0,
                    page_bytes_limit: self.page_bytes_limit as u32,
                    root,
                },
                written_page_count: 1,
                peak_buffer_bytes,
            });
        }

        let reference = self.store_page(CatalogPageBody::Leaf { entries: leaf })?;
        self.observe_streaming_page(observe, &reference)?;
        written_page_count = written_page_count.saturating_add(1);
        self.push_streaming_ref(
            &mut levels,
            &mut level_bytes,
            empty_branch_bytes,
            0,
            reference,
            &mut written_page_count,
            observe,
        )?;
        peak_buffer_bytes = peak_buffer_bytes.max(level_bytes.iter().copied().sum::<usize>());

        let (root_ref, height) = loop {
            let non_empty = levels
                .iter()
                .enumerate()
                .filter(|(_, level)| !level.is_empty())
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            if non_empty.len() == 1 && levels[non_empty[0]].len() == 1 {
                let height =
                    u16::try_from(non_empty[0]).map_err(|_| SegmentError::CorruptCatalog {
                        message: "catalog height exhausted u16".to_string(),
                    })?;
                break (
                    levels[non_empty[0]].pop().expect("one root reference"),
                    height,
                );
            }
            let level = *non_empty.first().expect("streaming catalog has references");
            let children = std::mem::take(&mut levels[level]);
            if children.len() < 2 {
                return Err(SegmentError::CorruptCatalog {
                    message: "streaming catalog would create a unary branch".to_string(),
                });
            }
            level_bytes[level] = empty_branch_bytes;
            let reference = self.store_page(CatalogPageBody::Branch { children })?;
            self.observe_streaming_page(observe, &reference)?;
            written_page_count = written_page_count.saturating_add(1);
            self.push_streaming_ref(
                &mut levels,
                &mut level_bytes,
                empty_branch_bytes,
                level + 1,
                reference,
                &mut written_page_count,
                observe,
            )?;
            peak_buffer_bytes = peak_buffer_bytes.max(level_bytes.iter().copied().sum::<usize>());
        };
        let root = CatalogRoot {
            format_version: CATALOG_FORMAT_VERSION,
            height,
            entry_count,
            page_bytes_limit: self.page_bytes_limit as u32,
            root: root_ref,
        };
        self.validate_root(&root)?;
        Ok(StreamingCatalogBuild {
            root,
            written_page_count,
            peak_buffer_bytes,
        })
    }

    fn push_streaming_ref<F>(
        &self,
        levels: &mut Vec<Vec<CatalogPageRef>>,
        level_bytes: &mut Vec<usize>,
        empty_branch_bytes: usize,
        level: usize,
        reference: CatalogPageRef,
        written_page_count: &mut u64,
        observe: &mut F,
    ) -> Result<()>
    where
        F: FnMut(&CatalogPageRef) -> Result<()>,
    {
        if levels.len() <= level {
            levels.resize_with(level + 1, Vec::new);
            level_bytes.resize(level + 1, empty_branch_bytes);
        }
        let encoded_reference =
            serde_json::to_vec(&reference).map_err(|error| SegmentError::Serialization {
                message: error.to_string(),
            })?;
        let added = encoded_reference.len() + usize::from(!levels[level].is_empty());
        if levels[level].is_empty()
            && level_bytes[level].saturating_add(added) > self.page_bytes_limit
        {
            return Err(SegmentError::CatalogPageTooLarge {
                limit: self.page_bytes_limit,
            });
        }
        if !levels[level].is_empty()
            && level_bytes[level].saturating_add(added) > self.page_bytes_limit
        {
            if levels[level].len() < 3 {
                return Err(SegmentError::CatalogPageTooLarge {
                    limit: self.page_bytes_limit,
                });
            }
            let mut children = std::mem::take(&mut levels[level]);
            let retained = children
                .pop()
                .expect("streaming level has at least three references");
            let parent = self.store_page(CatalogPageBody::Branch { children })?;
            self.observe_streaming_page(observe, &parent)?;
            let retained_bytes = serde_json::to_vec(&retained)
                .map_err(|error| SegmentError::Serialization {
                    message: error.to_string(),
                })?
                .len();
            levels[level].push(retained);
            level_bytes[level] = empty_branch_bytes.saturating_add(retained_bytes);
            *written_page_count = written_page_count.saturating_add(1);
            self.push_streaming_ref(
                levels,
                level_bytes,
                empty_branch_bytes,
                level + 1,
                parent,
                written_page_count,
                observe,
            )?;
        }
        level_bytes[level] = level_bytes[level]
            .saturating_add(encoded_reference.len() + usize::from(!levels[level].is_empty()));
        levels[level].push(reference);
        Ok(())
    }

    fn observe_streaming_page(
        &self,
        observe: &mut impl FnMut(&CatalogPageRef) -> Result<()>,
        reference: &CatalogPageRef,
    ) -> Result<()> {
        if let Err(error) = observe(reference) {
            self.store.delete(&reference.key)?;
            return Err(error);
        }
        Ok(())
    }

    pub fn upsert(&self, root: &CatalogRoot, entry: CatalogEntry) -> Result<CatalogMutation> {
        self.validate_root(root)?;
        self.validate_entry_key(&entry.key)?;
        let inserted = self.insert_page(&root.root, entry)?;
        let (root_ref, height) = if inserted.pages.len() == 1 {
            (inserted.pages[0].clone(), root.height)
        } else {
            let reference = self.store_page(CatalogPageBody::Branch {
                children: inserted.pages.clone(),
            })?;
            (
                reference,
                root.height
                    .checked_add(1)
                    .ok_or_else(|| SegmentError::CorruptCatalog {
                        message: "catalog height exhausted u16".to_string(),
                    })?,
            )
        };
        let mut written = inserted.written;
        if inserted.pages.len() > 1 {
            written.push(root_ref.key.clone());
        }
        let next = CatalogRoot {
            format_version: CATALOG_FORMAT_VERSION,
            height,
            entry_count: root
                .entry_count
                .checked_add(u64::from(inserted.inserted))
                .ok_or_else(|| SegmentError::CorruptCatalog {
                    message: "catalog entry count exhausted u64".to_string(),
                })?,
            page_bytes_limit: self.page_bytes_limit as u32,
            root: root_ref,
        };
        self.validate_root(&next)?;
        Ok(CatalogMutation {
            root: next,
            written_page_keys: written,
            obsolete_page_keys: inserted.obsolete,
        })
    }

    /// Remove one key with copy-on-write updates along only its search path.
    /// A missing key returns the original root without writing pages.
    pub fn remove(&self, root: &CatalogRoot, key: &str) -> Result<CatalogMutation> {
        self.validate_root(root)?;
        validate_catalog_key(key)?;
        let removed = self.remove_page(&root.root, key)?;
        if !removed.removed {
            return Ok(CatalogMutation {
                root: root.clone(),
                written_page_keys: Vec::new(),
                obsolete_page_keys: Vec::new(),
            });
        }
        let root_ref = if removed.pages.is_empty() {
            self.store_page(CatalogPageBody::Leaf {
                entries: Vec::new(),
            })?
        } else if removed.pages.len() == 1 {
            removed.pages[0].clone()
        } else {
            self.store_page(CatalogPageBody::Branch {
                children: removed.pages.clone(),
            })?
        };
        let mut written = removed.written;
        if removed.pages.len() != 1 {
            written.push(root_ref.key.clone());
        }
        let next = CatalogRoot {
            format_version: CATALOG_FORMAT_VERSION,
            height: if removed.pages.is_empty() {
                0
            } else if removed.pages.len() > 1 {
                root.height
                    .checked_add(1)
                    .ok_or_else(|| SegmentError::CorruptCatalog {
                        message: "catalog height exhausted u16".to_string(),
                    })?
            } else {
                root.height
            },
            entry_count: root.entry_count.checked_sub(1).ok_or_else(|| {
                SegmentError::CorruptCatalog {
                    message: "catalog entry count underflow".to_string(),
                }
            })?,
            page_bytes_limit: self.page_bytes_limit as u32,
            root: root_ref,
        };
        self.validate_root(&next)?;
        Ok(CatalogMutation {
            root: next,
            written_page_keys: written,
            obsolete_page_keys: removed.obsolete,
        })
    }

    pub fn lookup(&self, root: &CatalogRoot, key: &str) -> Result<Option<CatalogEntry>> {
        self.validate_root(root)?;
        validate_catalog_key(key)?;
        let mut reference = root.root.clone();
        loop {
            match self.load_page(&reference)? {
                CatalogPageBody::Leaf { entries } => {
                    return Ok(entries
                        .binary_search_by(|entry| entry.key.as_str().cmp(key))
                        .ok()
                        .map(|index| entries[index].clone()));
                }
                CatalogPageBody::Branch { children } => {
                    let index = child_index(&children, key)?;
                    reference = children[index].clone();
                }
            }
        }
    }

    /// Return the last entry in one lexicographic prefix with one tree-path
    /// read. Callers can keep exact per-partition high-water marks without a
    /// full catalog scan.
    pub fn last_with_prefix(
        &self,
        root: &CatalogRoot,
        prefix: &str,
    ) -> Result<Option<CatalogEntry>> {
        self.validate_root(root)?;
        validate_catalog_key(prefix)?;
        let candidate = match lexicographic_successor(prefix) {
            Some(upper) => self.last_before(root, &upper)?,
            None => self.last_entry(root)?,
        };
        Ok(candidate.filter(|entry| entry.key.starts_with(prefix)))
    }

    fn last_before(&self, root: &CatalogRoot, upper: &str) -> Result<Option<CatalogEntry>> {
        let mut reference = root.root.clone();
        loop {
            match self.load_page(&reference)? {
                CatalogPageBody::Leaf { entries } => {
                    let index = entries.partition_point(|entry| entry.key.as_str() < upper);
                    return Ok(index.checked_sub(1).map(|index| entries[index].clone()));
                }
                CatalogPageBody::Branch { children } => {
                    let index = children.partition_point(|child| child.first_key.as_str() < upper);
                    let Some(index) = index.checked_sub(1) else {
                        return Ok(None);
                    };
                    reference = children[index].clone();
                }
            }
        }
    }

    fn last_entry(&self, root: &CatalogRoot) -> Result<Option<CatalogEntry>> {
        let mut reference = root.root.clone();
        loop {
            match self.load_page(&reference)? {
                CatalogPageBody::Leaf { entries } => return Ok(entries.last().cloned()),
                CatalogPageBody::Branch { children } => {
                    reference = children.last().expect("validated branch").clone();
                }
            }
        }
    }

    pub fn reader(&self, root: &CatalogRoot) -> Result<CatalogReader> {
        self.validate_root(root)?;
        Ok(CatalogReader {
            catalog: self.clone(),
            pending: vec![root.root.clone()],
            leaf: VecDeque::new(),
            after_key: None,
            failed: false,
            peak_buffer_bytes: catalog_ref_bytes(&root.root),
        })
    }

    /// Stream entries strictly after `key` without reading earlier leaf pages.
    pub fn reader_after(&self, root: &CatalogRoot, key: &str) -> Result<CatalogReader> {
        self.validate_root(root)?;
        validate_catalog_key(key)?;
        Ok(CatalogReader {
            catalog: self.clone(),
            pending: vec![root.root.clone()],
            leaf: VecDeque::new(),
            after_key: Some(key.to_string()),
            failed: false,
            peak_buffer_bytes: catalog_ref_bytes(&root.root),
        })
    }

    pub fn page_keys(&self, root: &CatalogRoot) -> Result<CatalogPageKeyReader> {
        self.validate_root(root)?;
        Ok(CatalogPageKeyReader {
            catalog: self.clone(),
            pending: vec![root.root.clone()],
            failed: false,
        })
    }

    fn validate_root(&self, root: &CatalogRoot) -> Result<()> {
        if root.format_version != CATALOG_FORMAT_VERSION
            || root.page_bytes_limit as usize != self.page_bytes_limit
            || root.root.entry_count != root.entry_count
            || root.root.bytes == 0
            || root.root.bytes > self.page_bytes_limit as u64
            || root.root.sha256.len() != 64
        {
            return Err(SegmentError::CorruptCatalog {
                message: "root metadata is invalid".to_string(),
            });
        }
        Ok(())
    }

    fn insert_page(&self, reference: &CatalogPageRef, entry: CatalogEntry) -> Result<InsertPage> {
        match self.load_page(reference)? {
            CatalogPageBody::Leaf { mut entries } => {
                let (inserted, changed) =
                    match entries.binary_search_by(|current| current.key.cmp(&entry.key)) {
                        Ok(index) if entries[index] == entry => (false, false),
                        Ok(index) => {
                            entries[index] = entry;
                            (false, true)
                        }
                        Err(index) => {
                            entries.insert(index, entry);
                            (true, true)
                        }
                    };
                if !changed {
                    return Ok(InsertPage {
                        pages: vec![reference.clone()],
                        inserted,
                        written: Vec::new(),
                        obsolete: Vec::new(),
                    });
                }
                let mut pages = Vec::new();
                let mut written = Vec::new();
                for entries in self.pack_leaves(entries)? {
                    let page = self.store_page(CatalogPageBody::Leaf { entries })?;
                    written.push(page.key.clone());
                    pages.push(page);
                }
                Ok(InsertPage {
                    pages,
                    inserted,
                    written,
                    obsolete: vec![reference.key.clone()],
                })
            }
            CatalogPageBody::Branch { mut children } => {
                let index = child_index(&children, &entry.key)?;
                let child = self.insert_page(&children[index], entry)?;
                if child.written.is_empty() {
                    return Ok(InsertPage {
                        pages: vec![reference.clone()],
                        inserted: child.inserted,
                        written: Vec::new(),
                        obsolete: child.obsolete,
                    });
                }
                children.splice(index..=index, child.pages);
                let mut pages = Vec::new();
                let mut written = child.written;
                for children in self.pack_children(children)? {
                    let page = self.store_page(CatalogPageBody::Branch { children })?;
                    written.push(page.key.clone());
                    pages.push(page);
                }
                let mut obsolete = child.obsolete;
                obsolete.push(reference.key.clone());
                Ok(InsertPage {
                    pages,
                    inserted: child.inserted,
                    written,
                    obsolete,
                })
            }
        }
    }

    fn remove_page(&self, reference: &CatalogPageRef, key: &str) -> Result<RemovePage> {
        match self.load_page(reference)? {
            CatalogPageBody::Leaf { mut entries } => {
                let Ok(index) = entries.binary_search_by(|entry| entry.key.as_str().cmp(key))
                else {
                    return Ok(RemovePage {
                        pages: vec![reference.clone()],
                        removed: false,
                        written: Vec::new(),
                        obsolete: Vec::new(),
                    });
                };
                entries.remove(index);
                if entries.is_empty() {
                    return Ok(RemovePage {
                        pages: Vec::new(),
                        removed: true,
                        written: Vec::new(),
                        obsolete: vec![reference.key.clone()],
                    });
                }
                let mut pages = Vec::new();
                let mut written = Vec::new();
                for entries in self.pack_leaves(entries)? {
                    let page = self.store_page(CatalogPageBody::Leaf { entries })?;
                    written.push(page.key.clone());
                    pages.push(page);
                }
                Ok(RemovePage {
                    pages,
                    removed: true,
                    written,
                    obsolete: vec![reference.key.clone()],
                })
            }
            CatalogPageBody::Branch { mut children } => {
                let index = child_index(&children, key)?;
                if key < children[index].first_key.as_str()
                    || key > children[index].last_key.as_str()
                {
                    return Ok(RemovePage {
                        pages: vec![reference.clone()],
                        removed: false,
                        written: Vec::new(),
                        obsolete: Vec::new(),
                    });
                }
                let child = self.remove_page(&children[index], key)?;
                if !child.removed {
                    return Ok(RemovePage {
                        pages: vec![reference.clone()],
                        removed: false,
                        written: Vec::new(),
                        obsolete: child.obsolete,
                    });
                }
                children.splice(index..=index, child.pages);
                if children.is_empty() {
                    let mut obsolete = child.obsolete;
                    obsolete.push(reference.key.clone());
                    return Ok(RemovePage {
                        pages: Vec::new(),
                        removed: true,
                        written: child.written,
                        obsolete,
                    });
                }
                let mut pages = Vec::new();
                let mut written = child.written;
                for children in self.pack_children(children)? {
                    let page = self.store_page(CatalogPageBody::Branch { children })?;
                    written.push(page.key.clone());
                    pages.push(page);
                }
                let mut obsolete = child.obsolete;
                obsolete.push(reference.key.clone());
                Ok(RemovePage {
                    pages,
                    removed: true,
                    written,
                    obsolete,
                })
            }
        }
    }

    fn pack_leaves(&self, entries: Vec<CatalogEntry>) -> Result<Vec<Vec<CatalogEntry>>> {
        if entries.is_empty() {
            return Ok(vec![Vec::new()]);
        }
        let mut groups = Vec::new();
        let mut current = Vec::new();
        let empty_size = self.page_size(&CatalogPageBody::Leaf {
            entries: Vec::new(),
        })?;
        let mut current_size = empty_size;
        for entry in entries {
            let encoded =
                serde_json::to_vec(&entry).map_err(|error| SegmentError::Serialization {
                    message: error.to_string(),
                })?;
            let added = encoded.len() + usize::from(!current.is_empty());
            if !current.is_empty() && current_size.saturating_add(added) > self.page_bytes_limit {
                groups.push(std::mem::take(&mut current));
                current_size = empty_size;
            }
            let added = encoded.len() + usize::from(!current.is_empty());
            if current_size.saturating_add(added) > self.page_bytes_limit {
                return Err(SegmentError::CatalogPageTooLarge {
                    limit: self.page_bytes_limit,
                });
            }
            current_size = current_size.saturating_add(added);
            current.push(entry);
        }
        if !current.is_empty() {
            groups.push(current);
        }
        Ok(groups)
    }

    fn pack_children(&self, children: Vec<CatalogPageRef>) -> Result<Vec<Vec<CatalogPageRef>>> {
        if children.is_empty() {
            return Err(SegmentError::CorruptCatalog {
                message: "branch has no children".to_string(),
            });
        }
        let mut groups = Vec::new();
        let mut current = Vec::new();
        let empty_size = self.page_size(&CatalogPageBody::Branch {
            children: Vec::new(),
        })?;
        let mut current_size = empty_size;
        for child in children {
            let encoded =
                serde_json::to_vec(&child).map_err(|error| SegmentError::Serialization {
                    message: error.to_string(),
                })?;
            let added = encoded.len() + usize::from(!current.is_empty());
            if !current.is_empty() && current_size.saturating_add(added) > self.page_bytes_limit {
                groups.push(std::mem::take(&mut current));
                current_size = empty_size;
            }
            let added = encoded.len() + usize::from(!current.is_empty());
            if current_size.saturating_add(added) > self.page_bytes_limit {
                return Err(SegmentError::CatalogPageTooLarge {
                    limit: self.page_bytes_limit,
                });
            }
            current_size = current_size.saturating_add(added);
            current.push(child);
        }
        if !current.is_empty() {
            groups.push(current);
        }
        if groups.len() > 1 && groups.last().is_some_and(|group| group.len() == 1) {
            let only = groups
                .pop()
                .expect("a trailing singleton child group exists")
                .pop()
                .expect("the trailing child group has one item");
            let previous = groups
                .last_mut()
                .expect("a trailing singleton has a previous child group");
            if previous.len() < 3 {
                return Err(SegmentError::CatalogPageTooLarge {
                    limit: self.page_bytes_limit,
                });
            }
            let moved = previous
                .pop()
                .expect("the previous child group has at least three items");
            groups.push(vec![moved, only]);
        }
        Ok(groups)
    }

    fn page_size(&self, body: &CatalogPageBody) -> Result<usize> {
        encode_page(body).map(|bytes| bytes.len())
    }

    fn validate_entry_key(&self, key: &str) -> Result<()> {
        validate_catalog_key(key)?;
        let reference = CatalogPageRef {
            key: format!("{}/pages/{}.json", self.prefix, "0".repeat(64)),
            sha256: "0".repeat(64),
            bytes: u64::MAX,
            entry_count: u64::MAX,
            first_key: key.to_string(),
            last_key: key.to_string(),
        };
        if self.page_size(&CatalogPageBody::Branch {
            children: vec![reference.clone(), reference.clone(), reference],
        })? > self.page_bytes_limit
        {
            return Err(SegmentError::CatalogPageTooLarge {
                limit: self.page_bytes_limit,
            });
        }
        Ok(())
    }

    fn store_page(&self, body: CatalogPageBody) -> Result<CatalogPageRef> {
        validate_page_body(&body)?;
        let bytes = encode_page(&body)?;
        if bytes.len() > self.page_bytes_limit {
            return Err(SegmentError::CatalogPageTooLarge {
                limit: self.page_bytes_limit,
            });
        }
        let sha256 = hex_sha256(&bytes);
        let key = format!("{}/pages/{sha256}.json", self.prefix);
        match self
            .store
            .put(&key, &bytes, "application/json", PutCondition::IfAbsent)
        {
            Ok(_) => {}
            Err(ObjectStoreError::PreconditionFailed { .. }) => {
                let existing = self.store.get(&key)?;
                if existing.bytes != bytes {
                    return Err(SegmentError::ImmutableObjectChanged { key });
                }
            }
            Err(error) => return Err(error.into()),
        }
        let (entry_count, first_key, last_key) = page_bounds(&body)?;
        Ok(CatalogPageRef {
            key,
            sha256,
            bytes: bytes.len() as u64,
            entry_count,
            first_key,
            last_key,
        })
    }

    fn load_page(&self, reference: &CatalogPageRef) -> Result<CatalogPageBody> {
        let object = self.store.get(&reference.key)?;
        if object.bytes.len() as u64 != reference.bytes
            || hex_sha256(&object.bytes) != reference.sha256
            || object.bytes.len() > self.page_bytes_limit
        {
            return Err(SegmentError::CorruptCatalog {
                message: format!("page {} failed size or hash validation", reference.key),
            });
        }
        let page: CatalogPage =
            serde_json::from_slice(&object.bytes).map_err(|error| SegmentError::Serialization {
                message: error.to_string(),
            })?;
        if page.format_version != CATALOG_PAGE_FORMAT_VERSION {
            return Err(SegmentError::CorruptCatalog {
                message: format!("page {} has an unsupported format", reference.key),
            });
        }
        validate_page_body(&page.body)?;
        let (count, first, last) = page_bounds(&page.body)?;
        if count != reference.entry_count
            || first != reference.first_key
            || last != reference.last_key
        {
            return Err(SegmentError::CorruptCatalog {
                message: format!("page {} disagrees with its reference", reference.key),
            });
        }
        Ok(page.body)
    }
}

struct InsertPage {
    pages: Vec<CatalogPageRef>,
    inserted: bool,
    written: Vec<String>,
    obsolete: Vec<String>,
}

struct RemovePage {
    pages: Vec<CatalogPageRef>,
    removed: bool,
    written: Vec<String>,
    obsolete: Vec<String>,
}

pub struct CatalogReader {
    catalog: PagedCatalog,
    pending: Vec<CatalogPageRef>,
    leaf: VecDeque<CatalogEntry>,
    after_key: Option<String>,
    failed: bool,
    peak_buffer_bytes: usize,
}

pub struct CatalogPageKeyReader {
    catalog: PagedCatalog,
    pending: Vec<CatalogPageRef>,
    failed: bool,
}

impl Iterator for CatalogPageKeyReader {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        let reference = self.pending.pop()?;
        match self.catalog.load_page(&reference) {
            Ok(CatalogPageBody::Leaf { .. }) => Some(Ok(reference.key)),
            Ok(CatalogPageBody::Branch { children }) => {
                self.pending.extend(children.into_iter().rev());
                Some(Ok(reference.key))
            }
            Err(error) => {
                self.failed = true;
                Some(Err(error))
            }
        }
    }
}

impl CatalogReader {
    pub fn peak_buffer_bytes(&self) -> usize {
        self.peak_buffer_bytes
    }

    fn measure_buffer(&mut self, loaded_page_bytes: usize) {
        let pending = self.pending.iter().map(catalog_ref_bytes).sum::<usize>();
        let leaf = self
            .leaf
            .iter()
            .map(|entry| entry.key.len() + entry.value.len())
            .sum::<usize>();
        self.peak_buffer_bytes = self
            .peak_buffer_bytes
            .max(loaded_page_bytes.max(pending.saturating_add(leaf)));
    }
}

impl Iterator for CatalogReader {
    type Item = Result<CatalogEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        loop {
            if let Some(entry) = self.leaf.pop_front() {
                self.measure_buffer(0);
                return Some(Ok(entry));
            }
            let reference = self.pending.pop()?;
            if self
                .after_key
                .as_ref()
                .is_some_and(|after| reference.last_key <= *after)
            {
                continue;
            }
            let loaded_page_bytes = reference.bytes as usize;
            match self.catalog.load_page(&reference) {
                Ok(CatalogPageBody::Leaf { mut entries }) => {
                    if let Some(after) = &self.after_key {
                        let first = entries.partition_point(|entry| entry.key <= *after);
                        entries.drain(..first);
                    }
                    self.leaf = entries.into();
                }
                Ok(CatalogPageBody::Branch { children }) => {
                    self.pending.extend(
                        children
                            .into_iter()
                            .filter(|child| {
                                self.after_key
                                    .as_ref()
                                    .is_none_or(|after| child.last_key > *after)
                            })
                            .rev(),
                    );
                }
                Err(error) => {
                    self.failed = true;
                    return Some(Err(error));
                }
            }
            self.measure_buffer(loaded_page_bytes);
        }
    }
}

fn encode_page(body: &CatalogPageBody) -> Result<Vec<u8>> {
    serde_json::to_vec(&CatalogPage {
        format_version: CATALOG_PAGE_FORMAT_VERSION,
        body: body.clone(),
    })
    .map_err(|error| SegmentError::Serialization {
        message: error.to_string(),
    })
}

fn validate_catalog_key(key: &str) -> Result<()> {
    if key.is_empty() || key.len() > 1024 || key.contains('\0') {
        return Err(SegmentError::InvalidCatalogKey {
            key: key.to_string(),
        });
    }
    Ok(())
}

fn lexicographic_successor(value: &str) -> Option<String> {
    for (index, character) in value.char_indices().rev() {
        let mut scalar = u32::from(character).checked_add(1)?;
        if (0xd800..=0xdfff).contains(&scalar) {
            scalar = 0xe000;
        }
        if let Some(next) = char::from_u32(scalar) {
            let mut successor = value[..index].to_string();
            successor.push(next);
            return Some(successor);
        }
    }
    None
}

fn validate_page_body(body: &CatalogPageBody) -> Result<()> {
    match body {
        CatalogPageBody::Leaf { entries } => {
            let mut previous: Option<&str> = None;
            for entry in entries {
                validate_catalog_key(&entry.key)?;
                if previous.is_some_and(|key| key >= entry.key.as_str()) {
                    return Err(SegmentError::CorruptCatalog {
                        message: "leaf keys are not strictly sorted".to_string(),
                    });
                }
                previous = Some(&entry.key);
            }
        }
        CatalogPageBody::Branch { children } => {
            if children.is_empty() {
                return Err(SegmentError::CorruptCatalog {
                    message: "branch has no children".to_string(),
                });
            }
            let mut previous: Option<&str> = None;
            for child in children {
                if child.entry_count == 0
                    || child.first_key.is_empty()
                    || child.first_key > child.last_key
                    || previous.is_some_and(|key| key >= child.first_key.as_str())
                {
                    return Err(SegmentError::CorruptCatalog {
                        message: "branch child ranges are invalid".to_string(),
                    });
                }
                previous = Some(&child.last_key);
            }
        }
    }
    Ok(())
}

fn page_bounds(body: &CatalogPageBody) -> Result<(u64, String, String)> {
    match body {
        CatalogPageBody::Leaf { entries } => Ok((
            entries.len() as u64,
            entries
                .first()
                .map(|entry| entry.key.clone())
                .unwrap_or_default(),
            entries
                .last()
                .map(|entry| entry.key.clone())
                .unwrap_or_default(),
        )),
        CatalogPageBody::Branch { children } => Ok((
            children.iter().try_fold(0_u64, |count, child| {
                count
                    .checked_add(child.entry_count)
                    .ok_or_else(|| SegmentError::CorruptCatalog {
                        message: "catalog entry count exhausted u64".to_string(),
                    })
            })?,
            children
                .first()
                .expect("validated branch")
                .first_key
                .clone(),
            children.last().expect("validated branch").last_key.clone(),
        )),
    }
}

fn child_index(children: &[CatalogPageRef], key: &str) -> Result<usize> {
    if children.is_empty() {
        return Err(SegmentError::CorruptCatalog {
            message: "branch has no children".to_string(),
        });
    }
    Ok(children
        .partition_point(|child| child.last_key.as_str() < key)
        .min(children.len() - 1))
}

fn catalog_ref_bytes(reference: &CatalogPageRef) -> usize {
    std::mem::size_of::<CatalogPageRef>()
        + reference.key.len()
        + reference.sha256.len()
        + reference.first_key.len()
        + reference.last_key.len()
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
