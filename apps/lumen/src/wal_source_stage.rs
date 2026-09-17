//! Private durable staging for one in-memory WAL source.
//!
//! The source keeps its payload while this module streams it to a `StageStore`.
//! Only native `MemWal` may replace its source slot and issue the later RAM
//! release proof.  A staged handle pins this source's private directory until
//! the last handle is dropped.

use memmap2::Mmap;
use std::fs;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::change_admission::StagePayload;
use crate::committed_record_codec::{read_staged_wal_record, staged_generic_cbor_payload};
#[cfg(test)]
use crate::committed_stage::StageFailureInjector;
use crate::committed_stage::{DurableStage, SourceIdentity, SourceKind, StageStore};
use crate::storage::{Engine, RecordAdmissionError};
use crate::wal::WalRecord;

/// The bounded read buffer is charged by the caller together with the decoded
/// record before it asks this bridge to deserialize the record.
const READ_TRANSPORT_SCRATCH_BYTES: usize = 64 * 1024;
const CBOR_DECODE_SCRATCH_BYTES: usize = 4096;
const MEM_WAL_STAGE_EPOCH: u64 = 0;
static NEXT_MEM_WAL_SOURCE: AtomicU64 = AtomicU64::new(1);
static NEXT_STAGE_DIRECTORY: AtomicU64 = AtomicU64::new(0);

/// One process-private directory.  It is deliberately ephemeral: durable
/// receipt is a handoff within this process, never crash recovery policy.
struct StageDirectory {
    path: PathBuf,
}

impl StageDirectory {
    fn create() -> io::Result<Self> {
        Self::create_in(&std::env::temp_dir(), "lumen-mem-wal-stage")
    }

    fn create_in(parent: &Path, prefix: &str) -> io::Result<Self> {
        let process = std::process::id();
        for _ in 0..128 {
            let nonce = NEXT_STAGE_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("{prefix}-{process}-{nonce}"));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate a private MemWal stage directory",
        ))
    }
}

impl Drop for StageDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct SourceStageRoot {
    directory: StageDirectory,
    source_id: u64,
    source: SourceIdentity,
    #[cfg(test)]
    injector: Option<Arc<dyn StageFailureInjector>>,
}

/// Each receipt owns one child directory and removes only that directory on
/// its final drop. The parent source root remains available to other slots.
struct RecordStageRoot {
    directory: StageDirectory,
    store: StageStore,
}

/// A per-`MemWal` stager.  Clones share source identity and one private root.
#[derive(Clone)]
pub(crate) struct WalSourceStager {
    root: Arc<SourceStageRoot>,
}

/// Opaque durable receipt for a source WAL entry.  It deliberately carries no
/// RAM-release proof: successfully copying a record leaves the source slot
/// intact until `MemWal` performs its atomic replacement.
#[doc(hidden)]
pub(crate) struct StagedWalRecord {
    stage: DurableStage,
    source: Arc<SourceStageRoot>,
    record_root: Arc<RecordStageRoot>,
    sequence: u64,
    decoded_owned_bytes: usize,
    codec: StageCodec,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum StageCodec {
    Cbor,
    FastIndex,
}

/// Pins one validated native fast-Index source. The stage Arc retains the
/// private directory and the mmap retains the exact inode for a coordinator
/// capacity wait and later scalar apply.
pub(crate) struct MappedFastIndexPayload {
    _stage: Arc<StagedWalRecord>,
    mmap: Mmap,
}

/// Pins one validated generic-CBOR source. The mmap borrows only the private
/// `LWCS` body, and the stage Arc retains the private directory and inode.
pub(crate) struct MappedGenericCborPayload {
    _stage: Arc<StagedWalRecord>,
    mmap: Mmap,
    payload_start: usize,
}

impl MappedGenericCborPayload {
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.mmap[self.payload_start..]
    }
}

impl MappedFastIndexPayload {
    pub(crate) fn payload(&self) -> &[u8] {
        &self.mmap
    }
}

impl WalSourceStager {
    /// Create a distinct source identity and a private root for one `MemWal`.
    pub(crate) fn for_mem_wal() -> io::Result<Self> {
        let directory = StageDirectory::create()?;
        Self::with_directory(directory)
    }

    fn with_directory(directory: StageDirectory) -> io::Result<Self> {
        let source_id = NEXT_MEM_WAL_SOURCE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| io::Error::other("MemWal stage source identity exhausted"))?;
        let source = SourceIdentity::new(SourceKind::Local, format!("mem-wal:{source_id}"))?;
        Ok(Self {
            root: Arc::new(SourceStageRoot {
                directory,
                source_id,
                source,
                #[cfg(test)]
                injector: None,
            }),
        })
    }

    /// Stable for the process lifetime.  It has no relationship to a path or
    /// to a caller-supplied identifier.
    pub(crate) fn source_id(&self) -> u64 {
        self.root.source_id
    }

    /// Stream directly from the caller's record.  `StagePayload` serializes
    /// through the existing private CBOR codec and never builds a whole-record
    /// encoded buffer.  Errors return before this method changes `record`.
    pub(crate) fn stage(
        &self,
        sequence: u64,
        record: &mut WalRecord,
    ) -> io::Result<StagedWalRecord> {
        // Ciborium text growth, serde's temporary Content arrays and final
        // decoded values can overlap. Price those owners before writing the
        // source receipt; original String/Vec capacities alone are not enough.
        let decoded_owned_bytes =
            Engine::record_decode_peak(&record.entry).map_err(record_admission_io_error)?;
        let directory = StageDirectory::create_in(&self.root.directory.path, "record")?;
        #[cfg(test)]
        let store = match &self.root.injector {
            Some(injector) => StageStore::with_injector(&directory.path, injector.clone())?,
            None => StageStore::new(&directory.path)?,
        };
        #[cfg(not(test))]
        let store = StageStore::new(&directory.path)?;
        let record_root = Arc::new(RecordStageRoot { directory, store });
        let stage = record_root.store.stage_with_writer(
            sequence,
            MEM_WAL_STAGE_EPOCH,
            self.root.source.clone(),
            |output| record.write_stage(output),
        )?;
        Ok(StagedWalRecord {
            stage,
            source: self.root.clone(),
            record_root,
            sequence,
            decoded_owned_bytes,
            codec: StageCodec::Cbor,
        })
    }

    /// Stage only native v1 fast Index bytes. The caller retains its owned
    /// record until MemWal atomically swaps the slot after this returns.
    pub(crate) fn stage_fast_index(
        &self,
        sequence: u64,
        record: &WalRecord,
    ) -> io::Result<Option<StagedWalRecord>> {
        if !record.is_fast_index_wire() {
            return Ok(None);
        }
        let directory = StageDirectory::create_in(&self.root.directory.path, "record")?;
        #[cfg(test)]
        let store = match &self.root.injector {
            Some(injector) => StageStore::with_injector(&directory.path, injector.clone())?,
            None => StageStore::new(&directory.path)?,
        };
        #[cfg(not(test))]
        let store = StageStore::new(&directory.path)?;
        let record_root = Arc::new(RecordStageRoot { directory, store });
        let stage = record_root.store.stage_with_writer(
            sequence,
            MEM_WAL_STAGE_EPOCH,
            self.root.source.clone(),
            |output| record.write_fast_index_wire(output),
        )?;
        let decoded_owned_bytes =
            Engine::record_decode_peak(&record.entry).map_err(record_admission_io_error)?;
        Ok(Some(StagedWalRecord {
            stage,
            source: self.root.clone(),
            record_root,
            sequence,
            decoded_owned_bytes,
            codec: StageCodec::FastIndex,
        }))
    }

    #[cfg(test)]
    pub(crate) fn for_mem_wal_with_injector(
        injector: Arc<dyn StageFailureInjector>,
    ) -> io::Result<Self> {
        let mut stager = Self::for_mem_wal()?;
        Arc::get_mut(&mut stager.root)
            .expect("fresh stager root")
            .injector = Some(injector);
        Ok(stager)
    }
}

impl StagedWalRecord {
    pub(crate) fn source_id(&self) -> u64 {
        self.source.source_id
    }

    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Conservative owned bytes for the decoded `WalRecord`, calculated from
    /// the original source record before staging.
    pub(crate) fn decoded_owned_bytes(&self) -> usize {
        self.decoded_owned_bytes
    }

    /// Decode only after the caller has admitted the original decoded bound
    /// plus this fixed transport scratch.  This does not grant capacity or
    /// release the native WAL payload.
    pub(crate) fn read(&self, admitted_bytes: usize) -> io::Result<WalRecord> {
        let scratch = self.read_scratch_bytes();
        let required = self
            .decoded_owned_bytes
            .checked_add(scratch)
            .ok_or_else(|| io::Error::other("staged WAL read admission overflow"))?;
        if admitted_bytes < required {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "staged WAL record was read without decoded-record and transport admission",
            ));
        }
        if self.stage.sequence() != self.sequence
            || self.stage.engine_epoch() != MEM_WAL_STAGE_EPOCH
            || self.stage.source() != &self.source.source
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "staged WAL receipt does not belong to its MemWal source",
            ));
        }
        let file = self.stage.open()?;
        match self.codec {
            StageCodec::Cbor => {
                let mut reader =
                    BufReader::with_capacity(scratch - CBOR_DECODE_SCRATCH_BYTES, file);
                read_staged_wal_record(&mut reader)
            }
            StageCodec::FastIndex => {
                let mmap = unsafe { Mmap::map(&file)? };
                WalRecord::decode(&mmap)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))
            }
        }
    }

    pub(crate) fn read_scratch_bytes(&self) -> usize {
        self.decoded_owned_bytes.min(READ_TRANSPORT_SCRATCH_BYTES) + CBOR_DECODE_SCRATCH_BYTES
    }

    pub(crate) fn mapped_fast_index(
        self: &Arc<Self>,
    ) -> io::Result<Option<MappedFastIndexPayload>> {
        if self.codec != StageCodec::FastIndex {
            return Ok(None);
        }
        if self.stage.sequence() != self.sequence
            || self.stage.engine_epoch() != MEM_WAL_STAGE_EPOCH
            || self.stage.source() != &self.source.source
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "staged fast Index receipt does not belong to its MemWal source",
            ));
        }
        let file = self.stage.open()?;
        let mmap = unsafe { Mmap::map(&file)? };
        crate::wal::fast_index_scanner::FastIndexScanner::parse(&mmap)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        Ok(Some(MappedFastIndexPayload {
            _stage: self.clone(),
            mmap,
        }))
    }

    /// Map a private generic-CBOR stage without decoding its `WalRecord`.
    /// The stage identity is checked before opening it; `DurableStage::open`
    /// verifies the committed receipt before the mmap can lend its bytes.
    pub(crate) fn mapped_generic_cbor(
        self: &Arc<Self>,
    ) -> io::Result<Option<MappedGenericCborPayload>> {
        if self.codec != StageCodec::Cbor {
            return Ok(None);
        }
        if self.stage.sequence() != self.sequence
            || self.stage.engine_epoch() != MEM_WAL_STAGE_EPOCH
            || self.stage.source() != &self.source.source
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "staged generic CBOR receipt does not belong to its MemWal source",
            ));
        }
        let file = self.stage.open()?;
        let mmap = unsafe { Mmap::map(&file)? };
        let bytes = staged_generic_cbor_payload(&mmap)?;
        let payload_start = bytes.as_ptr() as usize - mmap.as_ptr() as usize;
        Ok(Some(MappedGenericCborPayload {
            _stage: self.clone(),
            mmap,
            payload_start,
        }))
    }

    #[cfg(test)]
    fn root_path(&self) -> &Path {
        &self.record_root.directory.path
    }
}

fn record_admission_io_error(error: RecordAdmissionError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::committed_stage::{StageFailureInjector, StageFailurePoint};
    use crate::log_entry::RaftLogEntry;
    use crate::types::{
        BatchUnindexDocsRequest, CreateCollectionRequest, FieldSpec, FieldType, FieldValue,
        IndexItem, IndexRequest, ReplaceDocItem, ReplaceDocsRequest,
    };
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    fn stager() -> WalSourceStager {
        WalSourceStager::for_mem_wal().unwrap()
    }

    fn read_admission(staged: &StagedWalRecord) -> usize {
        staged
            .decoded_owned_bytes()
            .checked_add(staged.read_scratch_bytes())
            .unwrap()
    }

    fn field_spec() -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Keyword,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    fn all_shapes() -> Vec<WalRecord> {
        let mut fields = BTreeMap::new();
        fields.insert("keyword".into(), field_spec());
        vec![
            WalRecord::new(RaftLogEntry::CreateCollection {
                collection_id: "orders".into(),
                req: CreateCollectionRequest { fields },
            }),
            WalRecord::new(RaftLogEntry::Index {
                collection_id: "orders".into(),
                req: IndexRequest {
                    request_id: Some("r".into()),
                    items: vec![
                        IndexItem {
                            external_id: "a".into(),
                            field: "string".into(),
                            value: FieldValue::String("value".into()),
                            version: Some(3),
                        },
                        IndexItem {
                            external_id: "b".into(),
                            field: "number".into(),
                            value: FieldValue::Number(f64::from_bits(0x7ff8_0000_0000_0123)),
                            version: None,
                        },
                        IndexItem {
                            external_id: "c".into(),
                            field: "vector".into(),
                            value: FieldValue::Vector(vec![
                                f32::from_bits(0x7fc0_0123),
                                f32::INFINITY,
                                f32::NEG_INFINITY,
                            ]),
                            version: None,
                        },
                        IndexItem {
                            external_id: "d".into(),
                            field: "set".into(),
                            value: FieldValue::StringList(vec!["x".into(), "y".into()]),
                            version: None,
                        },
                    ],
                },
            }),
            WalRecord::new(RaftLogEntry::ReplaceDocs {
                collection_id: "orders".into(),
                req: ReplaceDocsRequest {
                    docs: vec![ReplaceDocItem {
                        external_id: "a".into(),
                        version: Some(5),
                        fields: BTreeMap::new(),
                    }],
                },
            }),
            WalRecord::new(RaftLogEntry::TruncateDocs {
                collection_id: "orders".into(),
            }),
            WalRecord::new(RaftLogEntry::UnindexDocs {
                collection_id: "orders".into(),
                req: BatchUnindexDocsRequest {
                    external_ids: vec!["a".into()],
                },
            }),
            WalRecord::new(RaftLogEntry::Delete {
                collection_id: "orders".into(),
                external_id: "a".into(),
                field: Some("keyword".into()),
            }),
            WalRecord::new(RaftLogEntry::DropCollection {
                collection_id: "orders".into(),
                force: true,
            }),
            WalRecord::new(RaftLogEntry::AddField {
                collection_id: "orders".into(),
                field_name: "keyword".into(),
                spec: field_spec(),
            }),
            WalRecord::new(RaftLogEntry::DropField {
                collection_id: "orders".into(),
                field_name: "keyword".into(),
            }),
        ]
    }

    #[test]
    fn consumed_stage_reclaims_only_its_files_while_source_remains() {
        let stager = stager();
        let mut first = all_shapes().remove(0);
        let mut second = all_shapes().remove(1);
        let a = stager.stage(1, &mut first).unwrap();
        let b = stager.stage(2, &mut second).unwrap();
        let path_a = a.root_path().to_path_buf();
        let path_b = b.root_path().to_path_buf();
        drop(a);
        assert!(
            !path_a.exists(),
            "consumed source stage must release its files"
        );
        assert!(path_b.exists());
        assert!(b.read(read_admission(&b)).is_ok());
        let c = stager.stage(3, &mut first).unwrap();
        assert!(c.read(read_admission(&c)).is_ok());
    }

    // Append to wal_source_stage.rs tests. These cases intentionally use tightly
    // sized source vectors so any decoder growth is visible in the final estimate.
    fn tight<T>(mut values: Vec<T>) -> Vec<T> {
        values.shrink_to_fit();
        values
    }

    fn staged_bound(record: WalRecord) -> (usize, usize) {
        let stager = stager();
        let mut record = record;
        let staged = stager.stage(1, &mut record).unwrap();
        let bound = staged.decoded_owned_bytes();
        let decoded = staged.read(bound + staged.read_scratch_bytes()).unwrap();
        (bound, Engine::record_owned_bytes(&decoded.entry).unwrap())
    }

    #[test]
    fn staged_decode_never_exceeds_source_bound_for_sequence_shapes() {
        for count in [1, 33, 5001] {
            let items = tight(
                (0..count)
                    .map(|n| IndexItem {
                        external_id: format!("id-{n}"),
                        field: "v".into(),
                        value: FieldValue::String("x".into()),
                        version: None,
                    })
                    .collect(),
            );
            let (bound, decoded) = staged_bound(WalRecord::new(RaftLogEntry::Index {
                collection_id: "orders".into(),
                req: IndexRequest {
                    items,
                    request_id: None,
                },
            }));
            assert!(
                decoded <= bound,
                "Index items={count}: decoded={decoded} bound={bound}"
            );

            let values = tight((0..count).map(|n| format!("value-{n}")).collect());
            let (bound, decoded) = staged_bound(WalRecord::new(RaftLogEntry::Index {
                collection_id: "orders".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "id".into(),
                        field: "set".into(),
                        value: FieldValue::StringList(values),
                        version: None,
                    }],
                    request_id: None,
                },
            }));
            assert!(
                decoded <= bound,
                "StringList values={count}: decoded={decoded} bound={bound}"
            );

            let values = tight((0..count).map(|n| n as f32).collect());
            let (bound, decoded) = staged_bound(WalRecord::new(RaftLogEntry::Index {
                collection_id: "orders".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "id".into(),
                        field: "vector".into(),
                        value: FieldValue::Vector(values),
                        version: None,
                    }],
                    request_id: None,
                },
            }));
            assert!(
                decoded <= bound,
                "Vector values={count}: decoded={decoded} bound={bound}"
            );
        }
    }

    #[test]
    fn staged_decode_never_exceeds_source_bound_for_sparse_replacement_maps() {
        for count in [1, 33, 5001] {
            let fields = (0..count)
                .map(|n| (format!("field-{n}"), FieldValue::String("x".into())))
                .collect();
            let docs = tight(vec![ReplaceDocItem {
                external_id: "id".into(),
                version: None,
                fields,
            }]);
            let (bound, decoded) = staged_bound(WalRecord::new(RaftLogEntry::ReplaceDocs {
                collection_id: "orders".into(),
                req: ReplaceDocsRequest { docs },
            }));
            assert!(
                decoded <= bound,
                "Replace sparse fields={count}: decoded={decoded} bound={bound}"
            );
        }
    }

    #[test]
    fn stages_every_wal_shape_with_exact_version_and_nonfinite_bits() {
        let stager = stager();
        for (index, mut original) in all_shapes().into_iter().enumerate() {
            let mut expected = Vec::new();
            ciborium::ser::into_writer(&original, &mut expected).unwrap();
            let staged = stager.stage(index as u64 + 1, &mut original).unwrap();
            let decoded = staged.read(read_admission(&staged)).unwrap();
            let mut actual = Vec::new();
            ciborium::ser::into_writer(&decoded, &mut actual).unwrap();
            assert_eq!(actual, expected);
            assert_eq!(decoded.version, original.version);
        }
    }

    #[test]
    fn clones_share_one_stable_process_unique_source_id() {
        let first = stager();
        let clone = first.clone();
        let second = stager();
        assert_eq!(first.source_id(), clone.source_id());
        assert_ne!(first.source_id(), second.source_id());
    }

    #[test]
    fn staged_reader_keeps_private_root_after_stager_drops_then_last_drop_cleans() {
        let stager = stager();
        let mut record = all_shapes().remove(1);
        let staged = stager.stage(7, &mut record).unwrap();
        let path = staged.root_path().to_owned();
        assert!(path.is_dir());
        drop(stager);
        assert!(path.is_dir());
        assert!(staged.read(read_admission(&staged)).is_ok());
        drop(staged);
        assert!(!path.exists());
    }

    #[test]
    fn corrupt_staged_payload_is_refused_before_decode() {
        let stager = stager();
        let mut record = all_shapes().remove(0);
        let staged = stager.stage(9, &mut record).unwrap();
        let file = staged.root_path().join("records").join(format!(
            "{:020}-{:016x}.record",
            staged.sequence(),
            MEM_WAL_STAGE_EPOCH
        ));
        fs::write(file, b"corrupt").unwrap();
        assert_eq!(
            staged.read(read_admission(&staged)).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn native_fast_stage_streams_and_pins_a_validated_payload() {
        let stager = stager();
        let record = all_shapes().remove(1);
        let staged = Arc::new(stager.stage_fast_index(17, &record).unwrap().unwrap());
        let path = staged.root_path().to_owned();
        let mapped = staged.mapped_fast_index().unwrap().unwrap();
        assert_eq!(mapped.payload(), record.encode().unwrap());
        let scanner =
            crate::wal::fast_index_scanner::FastIndexScanner::parse(mapped.payload()).unwrap();
        assert_eq!(scanner.collection_id(), "orders");
        drop(staged);
        assert!(
            path.exists(),
            "mapped owner pins the private stage directory"
        );
        drop(mapped);
        assert!(!path.exists());
    }

    #[test]
    fn native_fast_stage_keeps_admitted_read_compatible_with_the_public_wire() {
        let stager = stager();
        let record = all_shapes().remove(1);
        let expected = record.encode().unwrap();
        let staged = stager.stage_fast_index(23, &record).unwrap().unwrap();
        assert!(staged.read(staged.decoded_owned_bytes()).is_err());
        let decoded = staged.read(read_admission(&staged)).unwrap();
        assert_eq!(decoded.encode().unwrap(), expected);
        assert_eq!(decoded.version, record.version);
    }

    #[test]
    fn private_generic_stage_maps_only_its_cbor_body_and_pins_the_receipt() {
        let stager = stager();
        let mut record = all_shapes().remove(0);
        let mut expected = Vec::new();
        ciborium::ser::into_writer(&record, &mut expected).unwrap();
        let staged = Arc::new(stager.stage(29, &mut record).unwrap());
        let path = staged.root_path().to_owned();
        let mapped = staged.mapped_generic_cbor().unwrap().unwrap();
        assert_eq!(mapped.bytes(), expected);
        drop(staged);
        assert!(
            path.exists(),
            "mapped owner pins the private stage directory"
        );
        drop(mapped);
        assert!(!path.exists());
    }

    #[test]
    fn private_generic_mapping_refuses_corrupt_envelope_or_payload_suffix() {
        for (name, mutate) in [
            (
                "header",
                Box::new(|bytes: &mut Vec<u8>| bytes[0] = b'X') as Box<dyn Fn(&mut Vec<u8>)>,
            ),
            ("version", Box::new(|bytes: &mut Vec<u8>| bytes[4] = 2)),
            ("cbor", Box::new(|bytes: &mut Vec<u8>| bytes.truncate(5))),
            ("suffix", Box::new(|bytes: &mut Vec<u8>| bytes.push(0))),
        ] {
            let stager = stager();
            let mut record = all_shapes().remove(0);
            let staged = Arc::new(stager.stage(30, &mut record).unwrap());
            let file = staged.root_path().join("records").join(format!(
                "{:020}-{:016x}.record",
                staged.sequence(),
                MEM_WAL_STAGE_EPOCH
            ));
            let mut bytes = fs::read(&file).unwrap();
            mutate(&mut bytes);
            fs::write(file, bytes).unwrap();
            assert!(staged.mapped_generic_cbor().is_err(), "{name}");
        }
    }

    #[test]
    fn dropping_one_stage_removes_only_its_child_directory() {
        let stager = stager();
        let mut first = all_shapes().remove(0);
        let mut second = all_shapes().remove(1);
        let first = stager.stage(41, &mut first).unwrap();
        let second = stager.stage(42, &mut second).unwrap();
        let first_path = first.root_path().to_owned();
        let second_path = second.root_path().to_owned();
        assert!(first_path.is_dir() && second_path.is_dir());
        drop(first);
        assert!(!first_path.exists());
        assert!(second_path.is_dir());
        assert!(second.read(read_admission(&second)).is_ok());
        drop(second);
        assert!(!second_path.exists());
    }

    struct FailAt(Mutex<Option<StageFailurePoint>>);
    impl StageFailureInjector for FailAt {
        fn check(&self, point: StageFailurePoint) -> io::Result<()> {
            let mut wanted = self.0.lock().unwrap();
            if *wanted == Some(point) {
                *wanted = None;
                return Err(io::Error::other("injected stage fault"));
            }
            Ok(())
        }
    }

    #[test]
    fn every_durable_stage_fault_preserves_the_caller_record() {
        for point in [
            StageFailurePoint::SyncPayload,
            StageFailurePoint::RenamePayload,
            StageFailurePoint::SyncPayloadDirectory,
            StageFailurePoint::SyncMarker,
            StageFailurePoint::PublishMarker,
            StageFailurePoint::SyncMarkerDirectory,
        ] {
            let stager = WalSourceStager::for_mem_wal_with_injector(Arc::new(FailAt(Mutex::new(
                Some(point),
            ))))
            .unwrap();
            let mut record = all_shapes().remove(1);
            let mut before = Vec::new();
            ciborium::ser::into_writer(&record, &mut before).unwrap();
            assert!(stager.stage(11, &mut record).is_err(), "{point:?}");
            let mut after = Vec::new();
            ciborium::ser::into_writer(&record, &mut after).unwrap();
            assert_eq!(after, before, "{point:?}");
        }
    }
}
