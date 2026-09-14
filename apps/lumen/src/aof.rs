// CODEGEN-BEGIN
//! Local append-only log (Stage 2 Phase 2f-3) — the binary's "AOF".
//!
//! The segment checkpoint ([`crate::segment_rdb`]) is the binary's "RDB": a
//! periodic, atomic snapshot of the materialized index, tagged with the WAL
//! sequence `S` it is current as of. Between checkpoints, this file is the
//! durable record of every APPLIED `(seq, WalRecord)` — the exact Redis
//! RDB+AOF split. Recovery is:
//!
//! 1. **RDB** — reopen the newest segment checkpoint (engine seeded to seq `S`),
//! 2. **AOF** — replay every frame with `seq > S` into the engine (to seq `A`),
//! 3. **Broker** — tail the log from `A + 1`.
//!
//! Because the AOF is durable through `A`, the broker stream only needs retention
//! beyond `A`, not from seq 0 — which is the whole point: broker retention can be
//! TRIMMED instead of kept forever.
//!
//! ## Frame format
//!
//! Each appended record is one self-describing frame:
//!
//! ```text
//! [ seq : u64 LE ][ len : u32 LE ][ crc : u32 LE ][ payload : len bytes ]
//! ```
//!
//! - `seq`     — the global sequence the record was applied at (the order key).
//! - `len`     — payload length in bytes.
//! - `crc`     — `crc32(payload)` (crc32fast), checked on replay.
//! - `payload` — the [`WalRecord`] encoded with ciborium (a compact, stable CBOR
//!   form — the same codec the segment checkpoint sidecars and CBOR RDB use).
//!
//! The 16-byte fixed header lets replay detect a TORN TAIL without parsing the
//! payload: if fewer than 16 header bytes remain, or `len` overruns EOF, or the
//! crc mismatches, the frame is incomplete (a crash landed mid-append) — replay
//! stops cleanly at the last good frame, with no panic and no error. The byte
//! offset of that last good frame's end is recorded so the next
//! [`AofWriter::open`] can truncate the torn tail before appending.
//!
//! ## fsync policy
//!
//! Mirrors Redis `appendfsync`:
//!
//! - [`FsyncPolicy::EverySec`] — append writes to the OS buffer; a
//!   periodic [`AofWriter::maybe_sync`] (call-driven, off the apply hot path)
//!   fsyncs at most once per second. A crash loses at most ~1s of un-fsynced
//!   tail, which replay recovers as a torn tail (the frames are still in the OS
//!   page cache up to the crash point, and any partial frame is discarded).
//! - [`FsyncPolicy::Always`] (default) — fsync after every append.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
pub use storage_durable::{FramedLogTrimObserver, FsyncPolicy};
use storage_durable::{FramedLogCursor, FramedLogWriter, LogFrame};
#[cfg(unix)]
use storage_durable::FramedLogTrimPlan;

#[cfg(test)]
use std::fs::OpenOptions;
#[cfg(test)]
use std::io::{Read, Seek, SeekFrom, Write};

use crate::wal::WalRecord;

/// Fixed per-frame header width: `seq(8) + len(4) + crc(4)`.
#[cfg(test)]
const HEADER_LEN: usize = 16;

/// Encode a [`WalRecord`] payload. Common high-QPS index records use Lumen's
/// fast binary WAL codec; uncommon records fall back to compact CBOR.
fn encode_payload(rec: &WalRecord) -> Result<Vec<u8>> {
    rec.encode().context("encode AOF record")
}

/// Decode an AOF payload back into a [`WalRecord`].
fn decode_payload(bytes: &[u8]) -> Result<WalRecord> {
    WalRecord::decode(bytes).context("decode AOF record")
}

/// Append-only writer keyed by applied seq. Frames are appended in seq order;
/// `open` first truncates any torn tail left by a crash mid-append, so the file
/// always starts in a clean, fully-decodable state.
pub struct AofWriter {
    inner: FramedLogWriter,
    #[cfg(test)]
    policy: FsyncPolicy,
    /// #2516: test-only ENOSPC fault injection, armed via
    /// [`AofWriter::set_inject_storage_full`]. Scoped to THIS writer instance
    /// (not a process-global flag) so parallel `cargo test` threads sharing
    /// the same test binary never cross-contaminate each other's AOF writes —
    /// each test opens its own `AofWriter` over its own tempdir.
    #[cfg(test)]
    inject_storage_full: std::sync::atomic::AtomicBool,
    /// Test-only one-shot non-ENOSPC failure. This models a single AOF gap
    /// followed by a healthy filesystem, so coordinator tests can prove that
    /// later records are still rejected after the first uncertain write.
    #[cfg(test)]
    inject_failure_once: Option<std::io::ErrorKind>,
}

/// A prepared durable AOF trim. It has no public configuration surface.
#[cfg(unix)]
pub(crate) struct AofTrimPlan {
    inner: FramedLogTrimPlan,
}

#[cfg(unix)]
impl AofTrimPlan {
    /// Copy and durably sync the stable trim prefix without borrowing the AOF
    /// writer, so `SharedAof` can admit normal appends during this work.
    pub(crate) fn copy_stable_prefix(&mut self) -> Result<()> {
        self.inner.copy_stable_prefix()
    }
}

impl AofWriter {
    /// Install an optional in-process observer for covered AOF frames and the
    /// initial temp-sync boundary during trim. This only forwards the shared
    /// framed-log hook; it does not change AOF locking, trimming, or durability.
    #[doc(hidden)]
    pub fn with_trim_observer(
        mut self,
        observer: std::sync::Arc<dyn FramedLogTrimObserver>,
    ) -> Self {
        self.inner = self.inner.with_trim_observer(observer);
        self
    }

    /// Begin a two-phase trim while the caller owns `SharedAof`.
    #[cfg(unix)]
    pub(crate) fn begin_trim(&mut self, through: u64) -> Result<AofTrimPlan> {
        self.inner
            .begin_trim_mapped(through)
            .map(|inner| AofTrimPlan { inner })
    }

    /// Finish a prepared trim while the caller owns `SharedAof`.
    #[cfg(unix)]
    pub(crate) fn finish_trim(&mut self, plan: AofTrimPlan) -> Result<()> {
        self.inner.finish_trim_mapped(plan.inner)
    }

    /// Open `path` for appending with the default [`FsyncPolicy::Always`],
    /// first truncating any torn tail.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Self::open_with_policy(path, FsyncPolicy::Always)
    }

    /// Open `path` for appending, first truncating any torn tail (a partial
    /// frame from a crash mid-append) so the next append lands after the last
    /// good frame. Creates the file (and parent dirs) if absent.
    pub fn open_with_policy(path: impl Into<PathBuf>, policy: FsyncPolicy) -> Result<Self> {
        let path = path.into();
        Ok(Self {
            inner: FramedLogWriter::open(&path, policy)?,
            #[cfg(test)]
            policy,
            #[cfg(test)]
            inject_storage_full: std::sync::atomic::AtomicBool::new(false),
            #[cfg(test)]
            inject_failure_once: None,
        })
    }

    /// #2516: arm/disarm the next typed or raw append call on THIS writer to
    /// fail with a synthetic `io::ErrorKind::StorageFull` error instead of
    /// touching the real file — the fault-injection seam that exercises the
    /// REAL production error-handling path (`crate::coordinator::is_storage_full`
    /// -> `Metrics::mark_storage_degraded` -> `crate::coordinator::StorageFullError`
    /// -> `ApiErr`'s 507 mapping) end to end without needing a genuinely full
    /// disk.
    #[cfg(test)]
    pub fn set_inject_storage_full(&self, on: bool) {
        self.inject_storage_full
            .store(on, std::sync::atomic::Ordering::SeqCst);
    }

    #[cfg(test)]
    pub fn inject_failure_once(&mut self, kind: std::io::ErrorKind) {
        self.inject_failure_once = Some(kind);
    }

    /// Append one applied `(seq, record)` frame. Buffered; durability follows the
    /// fsync policy (`Always` fsyncs now, `EverySec` defers to `maybe_sync`).
    pub fn append(&mut self, seq: u64, record: &WalRecord) -> Result<()> {
        self.check_injected_append_failure()?;
        let payload = encode_payload(record)?;
        self.append_large_payload(seq, &payload)
    }

    /// Append a payload which the caller already validated as one public WAL
    /// wire record. This keeps a mapped fast-Index source borrowed: it does not
    /// decode or clone its values. The coordinator owns wire validation before
    /// it calls this crate-private persistence primitive.
    pub(crate) fn append_raw_payload(&mut self, seq: u64, payload: &[u8]) -> Result<()> {
        self.check_injected_append_failure()?;
        self.append_large_payload(seq, payload)
    }

    fn check_injected_append_failure(&mut self) -> Result<()> {
        #[cfg(test)]
        if let Some(kind) = self.inject_failure_once.take() {
            return Err(anyhow::Error::new(std::io::Error::from(kind)));
        }
        #[cfg(test)]
        if self
            .inject_storage_full
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(anyhow::Error::new(std::io::Error::from(
                std::io::ErrorKind::StorageFull,
            )));
        }
        Ok(())
    }

    fn append_large_payload(&mut self, seq: u64, payload: &[u8]) -> Result<()> {
        self.inner.append_large_payload(seq, payload)
    }

    /// Flush the buffered writer to the OS (does NOT fsync). Cheap; safe to call
    /// often.
    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }

    /// Flush + fsync NOW, unconditionally. Resets the everysec timer.
    pub fn sync(&mut self) -> Result<()> {
        self.inner.sync()
    }

    /// Flush and fsync the AOF, then require a successful data-root directory
    /// fsync. Durable restore uses this before it can move `CURRENT`.
    pub fn sync_strict(&mut self) -> Result<()> {
        self.inner.sync_strict()
    }

    /// Call-driven everysec fsync: fsync only if dirty AND ≥ the cadence has
    /// elapsed since the last sync. Under `Always` this is a no-op (already
    /// synced on append). Meant to be called off the apply hot path (a periodic
    /// tick or after a batch), so the apply loop never blocks on fsync.
    pub fn maybe_sync(&mut self) -> Result<()> {
        self.inner.maybe_sync()
    }

    /// Drop every frame with `seq <= through`, keeping only newer frames. Called
    /// at checkpoint: once a checkpoint at seq `C` is durable in the segment RDB,
    /// every AOF frame with `seq <= C` is redundant and can be reclaimed.
    ///
    /// Crash-safe rewrite-survivors-to-temp + atomic rename: surviving frames are
    /// streamed (byte-for-byte, no re-encode) into `<path>.compact.tmp`, fsynced,
    /// then renamed over `path`. A crash before the rename leaves the original
    /// AOF intact (un-checkpointed frames are never lost); a crash after leaves
    /// the compacted AOF. The temp is removed first if a prior attempt left one.
    pub fn truncate_through(&mut self, through: u64) -> Result<()> {
        self.inner.truncate_through_mapped(through)
    }
}

/// Replay frames from an AOF, applying each `(seq, WalRecord)` with `seq >
/// from_seq` to a caller closure in order, stopping cleanly at a torn tail.
pub struct AofReader;

impl AofReader {
    /// Iterate every frame in `path` in order, SKIP frames with `seq <=
    /// from_seq` (already covered by the RDB baseline), and call `apply(seq,
    /// record)` for each frame with `seq > from_seq`. On a TORN TAIL (short read,
    /// `len` overruns EOF, or crc mismatch) STOP cleanly at the last good frame —
    /// no panic, no error. Returns the max seq REPLAYED (0 if none applied).
    ///
    /// An absent file replays nothing and returns 0 (a node that crashed before
    /// its first append simply has no AOF).
    pub fn replay(
        path: impl AsRef<Path>,
        from_seq: u64,
        apply: impl FnMut(u64, WalRecord),
    ) -> Result<u64> {
        let mut cursor = FramedLogCursor::open(path)?;
        replay_frames(from_seq, || cursor.next_frame(), apply)
    }
}

/// Replay one validated frame at a time in sequence order.
///
/// The next frame is fetched only after the previous callback returns.  This
/// keeps replay memory bounded by the frame currently being decoded.
fn replay_frames(
    from_seq: u64,
    mut next_frame: impl FnMut() -> Result<Option<LogFrame>>,
    mut apply: impl FnMut(u64, WalRecord),
) -> Result<u64> {
    let mut max_seq = 0u64;
    while let Some(frame) = next_frame()? {
        if frame.seq <= from_seq {
            continue;
        }
        let rec = decode_payload(&frame.payload)
            .with_context(|| format!("decode complete AOF frame at sequence {}", frame.seq))?;
        apply(frame.seq, rec);
        max_seq = max_seq.max(frame.seq);
    }
    Ok(max_seq)
}

/// Recovery helper: replay every AOF frame with `seq > from_seq` into `engine`
/// via [`crate::storage::Engine::apply_raft_entry`], returning the max seq
/// replayed. This is step 2 of cold start (RDB → **AOF** → broker); the engine is
/// already seeded to `from_seq` by the segment checkpoint.
pub fn replay_aof_into(
    engine: &std::sync::Arc<crate::storage::Engine>,
    path: impl AsRef<Path>,
    from_seq: u64,
) -> Result<u64> {
    // Initialize the capture cut before a relay can react to a process-global
    // waiter from another Engine. The observed helper repeats this idempotently.
    engine.capture_barrier.apply().initialize_sequence(from_seq);
    // The public replay owns its fallback before any admission can wait. The
    // observed helper below deliberately stays a lower-level manual seam.
    let mut capacity_owner = None;
    crate::segment_capacity::Fallback::ensure(&mut capacity_owner, engine, None)?;
    replay_aof_into_with_capacity_owner(engine, path, from_seq, || {}, &mut capacity_owner)
}

fn replay_aof_into_observed(
    engine: &std::sync::Arc<crate::storage::Engine>,
    path: impl AsRef<Path>,
    from_seq: u64,
    before_decode: impl FnMut(),
) -> Result<u64> {
    let mut capacity_owner = None;
    replay_aof_into_with_capacity_owner(
        engine,
        path,
        from_seq,
        before_decode,
        &mut capacity_owner,
    )
}

fn replay_aof_into_with_capacity_owner(
    engine: &std::sync::Arc<crate::storage::Engine>,
    path: impl AsRef<Path>,
    from_seq: u64,
    mut before_decode: impl FnMut(),
    capacity_owner: &mut Option<crate::segment_capacity::Fallback>,
) -> Result<u64> {
    engine.capture_barrier.apply().initialize_sequence(from_seq);
    let mut cursor = FramedLogCursor::open(path)?;
    let mut max_seq = 0u64;

    while let Some((start, frame)) = {
        let start = cursor.byte_offset();
        cursor
            .next_large_mapped_frame()?
            .map(|frame| (start, frame))
    } {
        if frame.seq <= from_seq {
            continue;
        }
        let seq = frame.seq;
        let encoded_bytes = frame.payload().len();
        let encoded_crc = crc32fast::hash(frame.payload());
        // A large fast Index frame remains mapped through its scalar projection.
        // Do this before scanner admission or generic decode, so one valid value
        // cannot become an owned record merely because recovery is replaying it.
        if encoded_bytes > crate::change_budget::HARD_LIMIT / 8 {
            if let Ok(scanner) =
                crate::wal::fast_index_scanner::FastIndexScanner::parse(frame.payload())
            {
                if engine.try_apply_committed_index_with_capacity_owner(&scanner, seq, || {
                    crate::segment_capacity::Fallback::ensure(capacity_owner, engine, None)
                }, |apply, outcome| {
                    if let Err(error) = outcome {
                        tracing::warn!(seq, error = %error, "AOF replay apply error (entry no-ops)");
                    }
                    apply.advance_sequence(seq);
                })? {
                    max_seq = max_seq.max(seq);
                    continue;
                }
            }
            if engine.try_apply_committed_replace_with_capacity_owner(
                frame.payload(), seq,
                &mut || crate::segment_capacity::Fallback::ensure(capacity_owner, engine, None),
                |apply, outcome| {
                    if let Err(error) = outcome {
                        tracing::warn!(seq, error = %error, "AOF replay replacement business error");
                    }
                    apply.advance_sequence(seq);
                },
            )? {
                max_seq = max_seq.max(seq);
                continue;
            }
        }
        // This first pass borrows mapped bytes and stores only token counters.
        // The source mapping is excluded from pending heap ownership. Reserve
        // before the typed scanner or decoder can allocate token buffers.
        let workspace = crate::wal_wire_cost::scan_workspace_bound(frame.payload())
            .with_context(|| format!("preflight complete AOF frame at sequence {seq}"))?;
        let request = engine.record_ram_request_from_bound(workspace, 0);
        let mut reservation = match engine.try_reserve_record_ram(&request) {
            Ok(reservation) => reservation,
            Err(crate::storage::RecordAdmissionError::Capacity(
                crate::change_budget::AdmissionError::Full { .. },
            )) => engine
                .wait_reserve_record_ram(&request)
                .context("wait for AOF replay scanner admission")?,
            Err(error) => {
                return Err(anyhow::Error::new(error).context("reserve AOF replay scanner"))
            }
        };
        let decoded_peak = crate::wal_wire_cost::decoded_peak_bound(frame.payload())
            .with_context(|| format!("price complete AOF frame at sequence {seq}"))?;
        match reservation.try_grow_to(decoded_peak) {
            Ok(()) => (),
            Err(crate::change_budget::AdmissionError::Full { .. }) => {
                engine.request_pending_checkpoint();
                reservation
                    .wait_grow_to(decoded_peak)
                    .map_err(crate::storage::RecordAdmissionError::Capacity)
                    .context("wait for AOF replay decode admission")?;
            }
            Err(error) => {
                return Err(
                    anyhow::Error::new(crate::storage::RecordAdmissionError::Capacity(error))
                        .context("reserve AOF replay decode"),
                )
            }
        }
        // The same cursor pins the original inode and open-time length across
        // capacity waits. Recheck that exact frame, never resolve a new path.
        let reread = cursor
            .reread_large_mapped_frame_at(start)
            .context("reread pinned AOF frame after admission")?
            .ok_or_else(|| anyhow::anyhow!("AOF frame vanished from pinned cursor"))?;
        anyhow::ensure!(
            reread.seq == seq
                && reread.payload().len() == encoded_bytes
                && crc32fast::hash(reread.payload()) == encoded_crc,
            "pinned AOF frame identity changed while replay waited"
        );
        before_decode();
        let record = decode_payload(reread.payload())
            .with_context(|| format!("decode complete AOF frame at sequence {seq}"))?;
        engine
            .price_decoded_record(&record.entry, &mut reservation, decoded_peak, 0, false)
            .context("retain decoded AOF record charge")?;
        drop(reread);
        drop(frame);
        // Decoder scratch is gone. Release its conservative allowance before
        // normalized changes are prepared; retain the actual decoded owner.
        reservation
            .release_transport_bytes()
            .map_err(crate::storage::RecordAdmissionError::Capacity)
            .context("release completed AOF decoder workspace")?;
        let mut entry = record.entry;
        loop {
            match engine.begin_admitted_record(entry, reservation) {
                Ok(mut prepared) => {
                    // Existing partial-prefix semantics retain a charge before
                    // dispatch and advance ordinary validation errors. Admission,
                    // preparation, decode, and reread failures returned above do
                    // not reach this point and therefore do not advance `seq`.
                    if let Err(error) = engine.apply_prepared_raft_entry(&mut prepared) {
                        tracing::warn!(seq, error = %error, "AOF replay apply error (entry no-ops)");
                    }
                    prepared.apply_lease().advance_sequence(seq);
                    max_seq = max_seq.max(seq);
                    break;
                }
                Err(mut reprice) => {
                    let Some(required) = reprice.required else {
                        return Err(
                            anyhow::Error::new(reprice.error).context("prepare AOF replay record")
                        );
                    };
                    // `begin_admitted_record` dropped its apply lease before it
                    // returned repricing ownership. Only missing normalized or
                    // staged bytes wait here; the decoded entry remains charged.
                    let grown = match reprice.reservation.try_grow_to(required) {
                        Err(crate::change_budget::AdmissionError::Full { .. }) => {
                            engine.request_pending_checkpoint();
                            reprice.reservation.wait_grow_to(required)
                        }
                        result => result,
                    };
                    grown
                        .map_err(crate::storage::RecordAdmissionError::Capacity)
                        .context("wait for AOF replay repricing")?;
                    entry = reprice.entry;
                    reservation = reprice.reservation;
                }
            }
        }
    }
    Ok(max_seq)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::VecDeque;

    #[test]
    fn open_uses_always_fsync_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let writer = AofWriter::open(dir.path().join("aof")).unwrap();
        assert_eq!(writer.policy, FsyncPolicy::Always);
    }

    #[test]
    fn strict_sync_makes_the_current_tail_replayable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aof");
        let mut writer = AofWriter::open(&path).unwrap();
        writer
            .append(
                1,
                &WalRecord::new(RaftLogEntry::DropCollection {
                    collection_id: "missing".into(),
                    force: true,
                }),
            )
            .unwrap();
        writer.sync_strict().unwrap();
        assert_eq!(AofReader::replay(&path, 0, |_, _| {}).unwrap(), 1);
    }
    use crate::change_budget::ChangeBudget;
    use crate::log_entry::RaftLogEntry;
    use crate::segment_rdb::SegmentRdbStore;
    use crate::storage::Engine;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
        MatchOp, MatchQuery, QueryNode, SearchRequest, TermQuery,
    };
    use std::collections::BTreeMap;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    fn create_entry(coll: &str) -> RaftLogEntry {
        RaftLogEntry::CreateCollection {
            collection_id: coll.into(),
            req: CreateCollectionRequest {
                fields: {
                    let mut f = BTreeMap::new();
                    f.insert(
                        "email".to_string(),
                        FieldSpec {
                            field_type: FieldType::Keyword,
                            analyzer: None,
                            multi: None,
                            dim: None,
                            metric: None,
                            backend: None,
                            quantize: None,
                        },
                    );
                    f
                },
            },
        }
    }

    fn index_entry(coll: &str, eid: &str, val: &str) -> RaftLogEntry {
        RaftLogEntry::Index {
            collection_id: coll.into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: eid.into(),
                    field: "email".into(),
                    value: FieldValue::String(val.into()),
                    version: None,
                }],
                request_id: None,
            },
        }
    }

    fn rec(entry: RaftLogEntry) -> WalRecord {
        WalRecord::new(entry)
    }

    /// Collect (seq, record-debug) by replaying with from_seq = 0.
    fn replay_seqs(path: &Path, from: u64) -> Vec<u64> {
        let mut out = Vec::new();
        AofReader::replay(path, from, |seq, _rec| out.push(seq)).unwrap();
        out
    }

    #[test]
    fn replay_reserves_capacity_before_decoding_a_complete_frame() {
        use crate::change_budget::ChangeBudget;
        use crate::storage::Engine;
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        use std::time::{Duration, Instant};

        const LIMIT: usize = 4 * 1024 * 1024;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aof.log");
        let budget = ChangeBudget::with_hard_limit(LIMIT);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine.apply_raft_entry(create_entry("u")).unwrap();
        {
            let mut writer = AofWriter::open(&path).unwrap();
            writer
                .append(1, &rec(index_entry("u", "id", &"x".repeat(512 * 1024))))
                .unwrap();
            writer.sync().unwrap();
        }
        let owner = budget.owner();
        let filler = owner.try_reserve(LIMIT - budget.snapshot().total).unwrap();
        let decodes = Arc::new(AtomicUsize::new(0));
        let replay_engine = engine.clone();
        let replay_decodes = decodes.clone();
        let replay = std::thread::spawn(move || {
            replay_aof_into_observed(&replay_engine, &path, 0, || {
                replay_decodes.fetch_add(1, Ordering::SeqCst);
            })
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while !budget.has_capacity_waiters() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        let waited = budget.has_capacity_waiters();
        let before_capacity = decodes.load(Ordering::SeqCst);
        // Always release the real budget owner and join replay before assertions.
        drop(filler);
        let completed = replay.join().unwrap().unwrap();
        assert!(waited, "replay must reach a real capacity wait");
        assert_eq!(
            before_capacity, 0,
            "AOF decoded a frame before pending capacity was reserved"
        );
        assert_eq!(completed, 1);
        assert_eq!(decodes.load(Ordering::SeqCst), 1);
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 1);
    }

    /// Write a valid frame whose payload is not a Lumen WAL record between two
    /// ordinary AOF records. `FramedLogCursor` can read all three frames only
    /// after it has checked each complete payload length and CRC.
    fn complete_crc_valid_malformed_middle(path: &Path) {
        {
            let mut writer = AofWriter::open_with_policy(path, FsyncPolicy::Always).unwrap();
            writer.append(1, &rec(create_entry("u"))).unwrap();
            writer.sync().unwrap();
        }
        {
            let mut writer = FramedLogWriter::open(path, FsyncPolicy::Always).unwrap();
            writer.append(2, b"not a Lumen WAL record").unwrap();
            writer.sync().unwrap();
        }
        {
            let mut writer = AofWriter::open_with_policy(path, FsyncPolicy::Always).unwrap();
            writer
                .append(
                    3,
                    &rec(index_entry("u", "after-malformed", "must-not-apply")),
                )
                .unwrap();
            writer.sync().unwrap();
        }
    }

    #[test]
    fn reader_rejects_complete_crc_valid_malformed_middle_frame_without_visiting_suffix() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("complete-malformed-middle.aof");
        complete_crc_valid_malformed_middle(&path);

        let mut cursor = FramedLogCursor::open(&path).unwrap();
        let first = cursor.next_frame().unwrap().unwrap();
        let middle = cursor.next_frame().unwrap().unwrap();
        let suffix = cursor.next_frame().unwrap().unwrap();
        assert_eq!((first.seq, middle.seq, suffix.seq), (1, 2, 3));
        assert!(decode_payload(&middle.payload).is_err());
        assert!(cursor.next_frame().unwrap().is_none());

        let mut visited = Vec::new();
        assert!(AofReader::replay(&path, 0, |seq, _| visited.push(seq)).is_err());
        assert_eq!(visited, vec![1]);
    }

    #[test]
    fn engine_replay_rejects_complete_crc_valid_malformed_middle_frame_at_prefix_watermark() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("complete-malformed-middle.aof");
        complete_crc_valid_malformed_middle(&path);
        let engine = Arc::new(Engine::new());

        assert!(replay_aof_into(&engine, &path, 0).is_err());
        let capture = engine.capture_barrier.capture(0).unwrap();
        assert_eq!(capture.stamp().sequence, 1);
        drop(capture);
        assert_eq!(
            engine
                .search("u", term_query("email", "must-not-apply"))
                .unwrap()
                .total,
            0
        );
    }

    #[test]
    fn replay_does_not_fetch_frame_two_before_applying_frame_one() {
        let frames = std::cell::RefCell::new(VecDeque::from([
            LogFrame {
                seq: 1,
                payload: encode_payload(&rec(create_entry("u"))).unwrap(),
            },
            LogFrame {
                seq: 2,
                payload: encode_payload(&rec(index_entry("u", "u1", "a@x"))).unwrap(),
            },
        ]));
        let polls = Cell::new(0usize);
        let mut applied = Vec::new();

        let max = replay_frames(
            0,
            || {
                polls.set(polls.get() + 1);
                Ok(frames.borrow_mut().pop_front())
            },
            |seq, _| {
                if seq == 1 {
                    assert_eq!(
                        polls.get(),
                        1,
                        "frame two must stay unread until frame one applies"
                    );
                }
                applied.push(seq);
            },
        )
        .unwrap();

        assert_eq!(applied, vec![1, 2]);
        assert_eq!(max, 2);
    }

    fn term_query(field: &str, value: &str) -> SearchRequest {
        SearchRequest {
            query: QueryNode::Term(TermQuery {
                field: field.into(),
                value: FieldValue::String(value.into()),
            }),
            limit: 10,
            offset: 0,
            cursor: None,
            routing_key: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn text_match_query(field: &str, text: &str) -> SearchRequest {
        SearchRequest {
            query: QueryNode::Match(MatchQuery {
                field: field.into(),
                text: text.into(),
                op: MatchOp::And,
            }),
            limit: 10,
            offset: 0,
            cursor: None,
            routing_key: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    #[test]
    fn replay_full_waits_for_real_checkpoint_then_replays_and_cold_recovers() {
        // Deliberately below CHECKPOINT_TRIGGER: replay itself must request a
        // checkpoint before its blocking reprice/wait, rather than depending on
        // the soft trigger.
        const HARD: usize = 64 * 1024;
        let budget = ChangeBudget::with_hard_limit(HARD);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        let RaftLogEntry::CreateCollection { req, .. } = create_entry("u") else {
            unreachable!()
        };
        engine.create_collection("u", req).unwrap();
        engine
            .index(
                "u",
                IndexRequest {
                    items: vec![IndexItem {
                        external_id: "filler".into(),
                        field: "email".into(),
                        value: FieldValue::String("x".repeat(12 * 1024)),
                        version: None,
                    }],
                    request_id: None,
                },
            )
            .unwrap();
        let entry = index_entry("u", "replayed", "after-checkpoint");
        let record = rec(entry);
        let payload = encode_payload(&record).unwrap();
        let raw = Engine::record_owned_bytes(&record.entry).unwrap();
        let active = budget.snapshot().total;
        let needed = raw.checked_add(payload.len()).unwrap();
        assert!(
            active + needed < HARD,
            "fixture must leave a reservable record"
        );
        let held = budget
            .owner()
            .try_reserve(HARD - active - needed + 1)
            .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let aof = dir.path().join("replay.aof");
        let mut writer = AofWriter::open(&aof).unwrap();
        writer.append(1, &record).unwrap();
        writer.sync().unwrap();
        let before = std::fs::read(&aof).unwrap();
        let wake = budget.checkpoint_wake();
        let observed = wake.epoch();
        let (done_tx, done_rx) = mpsc::channel();
        let replay_engine = engine.clone();
        let replay_path = aof.clone();
        std::thread::spawn(move || {
            done_tx
                .send(replay_aof_into(&replay_engine, replay_path, 0))
                .unwrap();
        });

        assert!(
            wake.wait_for_change_timeout(observed, Duration::from_secs(2)),
            "a full replay record must request a checkpoint before waiting"
        );
        assert!(
            done_rx.try_recv().is_err(),
            "replay must not apply before capacity releases"
        );
        assert_eq!(
            engine
                .search("u", term_query("email", "after-checkpoint"))
                .unwrap()
                .total,
            0
        );

        let store = SegmentRdbStore::new(dir.path().join("segments")).unwrap();
        store.save(&engine, 0).unwrap();
        assert_eq!(
            store.load_latest().unwrap().unwrap().1,
            0,
            "the blocked frame must not advance the checkpoint watermark"
        );
        assert_eq!(
            std::fs::read(&aof).unwrap(),
            before,
            "waiting never rewrites the AOF"
        );
        assert_eq!(
            done_rx
                .recv_timeout(Duration::from_secs(5))
                .unwrap()
                .unwrap(),
            1
        );
        assert_eq!(
            engine
                .search("u", term_query("email", "after-checkpoint"))
                .unwrap()
                .total,
            1
        );

        store.save(&engine, 1).unwrap();
        let (cold, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 1);
        assert_eq!(
            cold.search("u", term_query("email", "after-checkpoint"))
                .unwrap()
                .total,
            1
        );
        drop(held);
    }

    // Insert inside the existing `#[cfg(test)] mod tests` in apps/lumen/src/aof.rs,
    // after `replay_full_waits_for_real_checkpoint_then_replays_and_cold_recovers`.
    // It uses that module's existing imports and helpers: `ChangeBudget`, `Engine`,
    // `AofWriter`, `SegmentRdbStore`, `create_entry`, `index_entry`, `rec`, and
    // `term_query`.

    #[test]
    fn public_replay_full_checkpointable_work_starts_its_own_capacity_maintainer() {
        const HARD: usize = 64 * 1024;
        let budget = ChangeBudget::with_hard_limit(HARD);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        let RaftLogEntry::CreateCollection { req, .. } = create_entry("u") else {
            unreachable!()
        };
        engine.create_collection("u", req).unwrap();
        engine
            .index(
                "u",
                IndexRequest {
                    items: vec![IndexItem {
                        external_id: "checkpointable".into(),
                        field: "email".into(),
                        value: FieldValue::String("x".repeat(12 * 1024)),
                        version: None,
                    }],
                    request_id: None,
                },
            )
            .unwrap();
        let record = rec(index_entry("u", "replayed", "after-capacity-release"));
        let payload = encode_payload(&record).unwrap();
        let workspace = crate::wal_wire_cost::scan_workspace_bound(&payload).unwrap();
        let raw = Engine::record_owned_bytes(&record.entry).unwrap();
        let crate::change_record_cost::RecordEstimate::Ready(cost) =
            engine.estimate_record_cost(&record.entry)
        else {
            panic!("known Keyword fixture must have a normalized cost")
        };
        let normalized = raw + cost.active + cost.frozen + cost.prepublish;
        let active = budget.snapshot().total;
        assert!(active + workspace < HARD, "fixture scanner must fit after publication");
        let held_bytes = HARD - active - workspace + 1;
        let held = budget.owner().try_reserve(held_bytes).unwrap();
        let available = HARD - budget.snapshot().total;
        assert!(available < workspace, "fixture must block initial scanner admission");
        assert!(
            held_bytes + workspace <= HARD,
            "held charge and scanner workspace must fit after publication"
        );
        assert!(
            held_bytes + normalized <= HARD,
            "held charge and completed normalized record must fit after publication"
        );

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("standalone-progress.aof");
        let mut writer = AofWriter::open(&path).unwrap();
        writer.append(1, &record).unwrap();
        writer.sync().unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        let replay_engine = engine.clone();
        std::thread::spawn(move || {
            done_tx.send(replay_aof_into(&replay_engine, path, 0)).unwrap();
        });

        match done_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(result) => assert_eq!(result.unwrap(), 1),
            Err(timeout) => {
                // Cleanup only. The public replay is required to have arranged this
                // publication itself before the deadline above.
                let store = SegmentRdbStore::new(dir.path().join("cleanup-segments")).unwrap();
                store.save(&engine, 0).unwrap();
                assert_eq!(done_rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap(), 1);
                panic!("public replay did not start independent capacity maintenance: {timeout}");
            }
        }
        assert_eq!(
            engine.search("u", term_query("email", "after-capacity-release")).unwrap().total,
            1
        );
        drop(held);
    }

    #[test]
    fn public_replay_growth_wait_starts_its_own_capacity_maintainer() {
        const HARD: usize = 256 * 1024;
        let budget = ChangeBudget::with_hard_limit(HARD);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        let RaftLogEntry::CreateCollection { req, .. } = create_entry("u") else {
            unreachable!()
        };
        engine.create_collection("u", req).unwrap();
        engine
            .index(
                "u",
                IndexRequest {
                    items: vec![IndexItem {
                        external_id: "checkpointable".into(),
                        field: "email".into(),
                        value: FieldValue::String("x".repeat(12 * 1024)),
                        version: None,
                    }],
                    request_id: None,
                },
            )
            .unwrap();
        let keyword = FieldSpec {
            field_type: FieldType::Keyword,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        };
        let fields = (0..8)
            .map(|ordinal| (format!("field-{ordinal:03}"), keyword.clone()))
            .collect();
        let record = rec(RaftLogEntry::CreateCollection {
            collection_id: "growth-created".into(),
            req: CreateCollectionRequest { fields },
        });
        let payload = encode_payload(&record).unwrap();
        let workspace = crate::wal_wire_cost::scan_workspace_bound(&payload).unwrap();
        let decoded = crate::wal_wire_cost::decoded_peak_bound(&payload).unwrap();
        assert!(workspace < decoded, "fixture must pass scanner reserve before decoded growth");
        let raw = Engine::record_owned_bytes(&record.entry).unwrap();
        let crate::change_record_cost::RecordEstimate::Ready(cost) =
            engine.estimate_record_cost(&record.entry)
        else {
            panic!("CreateCollection fixture must have a normalized cost")
        };
        let normalized = raw + cost.active + cost.frozen + cost.prepublish;
        let active = budget.snapshot().total;
        assert!(active + decoded < HARD, "fixture decoded record must fit after publication");
        let held_bytes = HARD - active - decoded + 1;
        let held = budget.owner().try_reserve(held_bytes).unwrap();
        let available = HARD - budget.snapshot().total;
        assert!(workspace <= available, "fixture scanner reserve must fit");
        assert!(available < decoded, "fixture must block only decoded growth");
        assert!(
            held_bytes + normalized <= HARD,
            "held charge and completed normalized record must fit after publication"
        );

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("standalone-growth.aof");
        let mut writer = AofWriter::open(&path).unwrap();
        writer.append(1, &record).unwrap();
        writer.sync().unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        let replay_engine = engine.clone();
        std::thread::spawn(move || {
            done_tx.send(replay_aof_into(&replay_engine, path, 0)).unwrap();
        });

        match done_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(result) => assert_eq!(result.unwrap(), 1),
            Err(timeout) => {
                let store = SegmentRdbStore::new(dir.path().join("cleanup-segments")).unwrap();
                store.save(&engine, 0).unwrap();
                assert_eq!(done_rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap(), 1);
                panic!("public replay decoded-growth wait had no capacity maintainer: {timeout}");
            }
        }
        assert!(engine.list_collections().unwrap().contains(&"growth-created".to_owned()));
        drop(held);
    }

    #[test]
    fn public_replay_does_not_checkpoint_reserved_only_capacity_pressure() {
        const HARD: usize = 64 * 1024;
        let budget = ChangeBudget::with_hard_limit(HARD);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        let RaftLogEntry::CreateCollection { req, .. } = create_entry("u") else {
            unreachable!()
        };
        engine.create_collection("u", req).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let baseline = SegmentRdbStore::new(dir.path().join("baseline-segments")).unwrap();
        baseline.save(&engine, 0).unwrap();
        assert_eq!(budget.snapshot().total, 0, "baseline checkpoint must release schema setup work");
        let record = rec(index_entry("u", "replayed", "after-reservation-release"));
        let payload = encode_payload(&record).unwrap();
        let workspace = crate::wal_wire_cost::scan_workspace_bound(&payload).unwrap();
        let decoded = crate::wal_wire_cost::decoded_peak_bound(&payload).unwrap();
        assert!(workspace > 0, "fixture must have scanner admission work");
        assert!(decoded < HARD, "fixture must be fitting, rather than oversized");
        let held = budget.owner().try_reserve(HARD - workspace + 1).unwrap();
        let checkpoints_before = engine.metrics().segment_checkpoint_completed_total.get();

        let path = dir.path().join("reserved-only.aof");
        let mut writer = AofWriter::open(&path).unwrap();
        writer.append(1, &record).unwrap();
        writer.sync().unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        let replay_engine = engine.clone();
        std::thread::spawn(move || {
            done_tx.send(replay_aof_into(&replay_engine, path, 0)).unwrap();
        });

        let waiter_deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !budget.has_capacity_waiters() && std::time::Instant::now() < waiter_deadline {
            std::thread::yield_now();
        }
        assert!(budget.has_capacity_waiters(), "replay must reach the real initial reservation wait");
        assert!(
            done_rx.recv_timeout(Duration::from_millis(100)).is_err(),
            "reserved-only capacity must not be treated as checkpointable progress"
        );
        assert_eq!(
            engine.metrics().segment_checkpoint_completed_total.get(),
            checkpoints_before,
            "reserved-only pressure must not publish a pointless checkpoint"
        );
        drop(held);
        assert_eq!(done_rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap(), 1);
        assert_eq!(
            engine
                .search("u", term_query("email", "after-reservation-release"))
                .unwrap()
                .total,
            1
        );
    }
    #[test]
    fn replay_reprice_with_free_capacity_does_not_request_checkpoint() {
        let budget = ChangeBudget::with_hard_limit(64 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine.apply_raft_entry(create_entry("u")).unwrap();
        let RaftLogEntry::Index { req, .. } = index_entry("u", "existing", "kept") else {
            unreachable!()
        };
        engine.index("u", req).unwrap();
        assert!(
            budget.snapshot().active > 0,
            "fixture needs checkpointable preceding work"
        );
        let value = "a".repeat(2048);
        let record = rec(index_entry("u", "replayed", &value));
        let raw = Engine::record_owned_bytes(&record.entry).unwrap();
        let crate::change_record_cost::RecordEstimate::Ready(cost) =
            engine.estimate_record_cost(&record.entry)
        else {
            panic!("Keyword fixture must have a known normalized cost")
        };
        let normalized = raw + cost.active + cost.frozen + cost.prepublish;
        assert!(normalized > raw, "fixture must need normalized repricing");
        assert!(budget.snapshot().total + normalized < 64 * 1024);
        let before = budget.snapshot().checkpoint_request_revision;
        assert_eq!(before, None);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reprice-with-room.aof");
        let mut writer = AofWriter::open(&path).unwrap();
        writer.append(1, &record).unwrap();
        writer.sync().unwrap();
        assert_eq!(replay_aof_into(&engine, &path, 0).unwrap(), 1);
        assert_eq!(
            engine
                .search("u", term_query("email", &value))
                .unwrap()
                .total,
            1
        );
        assert_eq!(
            budget.snapshot().checkpoint_request_revision,
            before,
            "repricing that fits must not force a checkpoint of the preceding record"
        );
    }

    #[test]
    fn replay_reprice_error_keeps_failed_frame_and_watermark() {
        // This is deliberately an unresolved oversized prepared-Text record.
        // The ordinary replay route must return Err before it mutates or
        // advances, rather than using the old uncharged apply fallback.
        let budget = ChangeBudget::with_hard_limit(64 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget));
        engine
            .create_collection(
                "text",
                CreateCollectionRequest {
                    fields: BTreeMap::from([(
                        "body".into(),
                        FieldSpec {
                            field_type: FieldType::Text,
                            analyzer: Some(crate::types::Analyzer::Ngram),
                            multi: None,
                            dim: None,
                            metric: None,
                            backend: None,
                            quantize: None,
                        },
                    )]),
                },
            )
            .unwrap();
        // Repetitions now admit their distinct terms. Keep this refusal
        // fixture truly oversized even when its terms are priced exactly.
        let value = format!("ab{}", (0x4e00..0x4e00 + 512)
            .map(|scalar| char::from_u32(scalar).unwrap()).collect::<String>());
        let mut distinct = std::collections::BTreeSet::new();
        crate::ngram_stream::stream_default_ngrams(&value, |token| {
            distinct.insert(token.to_owned());
            Ok::<_, ()>(())
        }).unwrap();
        let normalized = crate::change_memory_cost::estimate_change(
            &crate::change_memory_cost::Change::Index {
                external_id_bytes: "not-applied".len(),
                new_document: true,
                field: crate::change_memory_cost::FieldCost::Text {
                    distinct_terms: distinct.len(),
                    total_term_bytes: distinct.iter().map(String::len).sum(),
                },
                volatile_metadata_bytes: 0,
            },
        ).unwrap();
        assert!(normalized.total() > 64 * 1024,
            "distinct normalized data must exceed this test's budget before raw transport is added");
        let record = rec(RaftLogEntry::Index {
            collection_id: "text".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "not-applied".into(),
                    field: "body".into(),
                    value: FieldValue::String(value),
                    version: None,
                }],
                request_id: None,
            },
        });
        let dir = tempfile::tempdir().unwrap();
        let aof = dir.path().join("reprice.aof");
        let mut writer = AofWriter::open(&aof).unwrap();
        writer.append(1, &record).unwrap();
        writer.sync().unwrap();
        let before = std::fs::read(&aof).unwrap();

        assert!(replay_aof_into(&engine, &aof, 0).is_err());
        assert_eq!(
            std::fs::read(&aof).unwrap(),
            before,
            "failed replay preserves its source"
        );
        assert_eq!(
            engine
                .search("text", text_match_query("body", "ab"))
                .unwrap()
                .total,
            0
        );
        let store = SegmentRdbStore::new(dir.path().join("segments")).unwrap();
        store.save(&engine, 0).unwrap();
        assert_eq!(store.load_latest().unwrap().unwrap().1, 0);
        assert_eq!(
            AofReader::replay(&aof, 0, |seq, _| assert_eq!(seq, 1)).unwrap(),
            1
        );
    }

    #[test]
    fn append_then_replay_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        w.append(1, &rec(create_entry("u"))).unwrap();
        w.append(2, &rec(index_entry("u", "u1", "a@x.com")))
            .unwrap();
        w.append(3, &rec(index_entry("u", "u2", "b@x.com")))
            .unwrap();
        w.sync().unwrap();

        let mut seqs = Vec::new();
        let mut kinds = Vec::new();
        let max = AofReader::replay(&path, 0, |seq, r| {
            seqs.push(seq);
            kinds.push(matches!(r.entry, RaftLogEntry::CreateCollection { .. }));
        })
        .unwrap();
        assert_eq!(seqs, vec![1, 2, 3]);
        assert_eq!(max, 3);
        assert_eq!(kinds, vec![true, false, false]);
    }

    #[test]
    fn append_raw_payload_keeps_the_validated_wire_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("raw.aof");
        let payload = rec(index_entry("u", "u1", "raw@x")).encode().unwrap();
        let mut writer = AofWriter::open(&path).unwrap();
        writer.append_raw_payload(9, &payload).unwrap();
        writer.sync().unwrap();
        let mut cursor = FramedLogCursor::open(&path).unwrap();
        let frame = cursor.next_frame().unwrap().unwrap();
        assert_eq!(frame.seq, 9);
        assert_eq!(frame.payload, payload);
    }

    #[test]
    fn append_raw_payload_uses_the_existing_storage_full_refusal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("raw-refusal.aof");
        let payload = rec(index_entry("u", "u1", "raw@x")).encode().unwrap();
        let mut writer = AofWriter::open(&path).unwrap();
        writer.set_inject_storage_full(true);
        assert!(writer.append_raw_payload(1, &payload).is_err());
        writer.set_inject_storage_full(false);
        assert_eq!(AofReader::replay(&path, 0, |_, _| {}).unwrap(), 0);
    }

    #[test]
    fn replay_non_fast_record_uses_the_legacy_decode_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.aof");
        let mut writer = AofWriter::open(&path).unwrap();
        let record = WalRecord {
            version: crate::wal::WAL_FORMAT_VERSION,
            entry: RaftLogEntry::DropCollection {
                collection_id: "missing".into(),
                force: true,
            },
        };
        writer.append(1, &record).unwrap();
        writer.sync().unwrap();
        let engine = Arc::new(Engine::new());
        let decoded = Cell::new(0);
        assert_eq!(
            replay_aof_into_observed(&engine, &path, 0, || decoded.set(decoded.get() + 1)).unwrap(),
            1
        );
        assert_eq!(decoded.get(), 1);
    }

    #[test]
    fn replay_skips_at_or_below_from_seq() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        for s in 1..=5 {
            w.append(s, &rec(index_entry("u", &format!("u{s}"), "x@y")))
                .unwrap();
        }
        w.sync().unwrap();
        // from_seq = 3 → only seq 4, 5 are replayed (strict `>`).
        assert_eq!(replay_seqs(&path, 3), vec![4, 5]);
        // from_seq = 0 → all.
        assert_eq!(replay_seqs(&path, 0), vec![1, 2, 3, 4, 5]);
        // from_seq = 5 → none.
        assert_eq!(replay_seqs(&path, 5), Vec::<u64>::new());
    }

    #[test]
    fn truncate_through_keeps_only_newer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        for s in 1..=6 {
            w.append(s, &rec(index_entry("u", &format!("u{s}"), "x@y")))
                .unwrap();
        }
        w.sync().unwrap();
        w.truncate_through(4).unwrap();
        // Frames 1..=4 dropped; 5, 6 survive.
        assert_eq!(replay_seqs(&path, 0), vec![5, 6]);
        // And the re-opened append handle keeps appending after the survivors.
        w.append(7, &rec(index_entry("u", "u7", "x@y"))).unwrap();
        w.sync().unwrap();
        assert_eq!(replay_seqs(&path, 0), vec![5, 6, 7]);
    }

    #[cfg(unix)]
    #[test]
    fn two_phase_trim_wrapper_retains_the_late_suffix_and_busy_plan() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("two-phase.aof");
        let mut writer = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        writer
            .append(1, &rec(index_entry("u", "covered", "one@example.test")))
            .unwrap();
        writer
            .append(2, &rec(index_entry("u", "retained", "two@example.test")))
            .unwrap();

        let mut plan = writer.begin_trim(1).unwrap();
        assert!(writer.begin_trim(1).is_err(), "an active plan must stay owned");
        plan.copy_stable_prefix().unwrap();
        writer
            .append(3, &rec(index_entry("u", "late", "three@example.test")))
            .unwrap();
        writer.finish_trim(plan).unwrap();

        assert_eq!(replay_seqs(&path, 0), vec![2, 3]);
    }

    #[test]
    fn truncate_through_survivors_persist_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        {
            let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
            for s in 1..=5 {
                w.append(s, &rec(index_entry("u", &format!("u{s}"), "x@y")))
                    .unwrap();
            }
            w.sync().unwrap();
            w.truncate_through(2).unwrap();
        }
        // A fresh open sees only the survivors and can extend them.
        let mut w2 = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        w2.append(6, &rec(index_entry("u", "u6", "x@y"))).unwrap();
        w2.sync().unwrap();
        assert_eq!(replay_seqs(&path, 0), vec![3, 4, 5, 6]);
    }

    #[test]
    fn torn_tail_replays_prefix_then_recovers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        w.append(1, &rec(create_entry("u"))).unwrap();
        w.append(2, &rec(index_entry("u", "u1", "a@x"))).unwrap();
        w.append(3, &rec(index_entry("u", "u2", "b@x"))).unwrap();
        w.sync().unwrap();

        // Simulate a crash mid-append: corrupt the tail by appending a partial,
        // garbage frame (a header claiming a length that overruns EOF).
        let good_len = std::fs::metadata(&path).unwrap().len();
        {
            use std::io::Write as _;
            let mut f = OpenOptions::new().append(true).open(&path).unwrap();
            // seq=99, len=1_000_000 (way past EOF), crc=0, then a single byte.
            let mut hdr = [0u8; HEADER_LEN];
            hdr[0..8].copy_from_slice(&99u64.to_le_bytes());
            hdr[8..12].copy_from_slice(&1_000_000u32.to_le_bytes());
            f.write_all(&hdr).unwrap();
            f.write_all(&[0xAB]).unwrap();
            f.sync_all().unwrap();
        }
        assert!(std::fs::metadata(&path).unwrap().len() > good_len);

        // Replay stops cleanly at the last good frame — no panic, no error.
        assert_eq!(replay_seqs(&path, 0), vec![1, 2, 3]);

        // The next open TRUNCATES the torn tail back to the last good frame.
        let mut w2 = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        assert_eq!(std::fs::metadata(&path).unwrap().len(), good_len);
        // And a fresh append lands right after frame 3.
        w2.append(4, &rec(index_entry("u", "u3", "c@x"))).unwrap();
        w2.sync().unwrap();
        assert_eq!(replay_seqs(&path, 0), vec![1, 2, 3, 4]);
    }

    #[test]
    fn torn_tail_via_crc_mismatch_stops_at_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.aof");
        let mut w = AofWriter::open_with_policy(&path, FsyncPolicy::Always).unwrap();
        w.append(1, &rec(index_entry("u", "u1", "a@x"))).unwrap();
        w.append(2, &rec(index_entry("u", "u2", "b@x"))).unwrap();
        w.sync().unwrap();

        // Flip a byte in the LAST frame's payload → crc mismatch → torn tail.
        let len = std::fs::metadata(&path).unwrap().len();
        {
            let mut f = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .unwrap();
            f.seek(SeekFrom::Start(len - 1)).unwrap();
            let mut b = [0u8; 1];
            f.read_exact(&mut b).unwrap();
            f.seek(SeekFrom::Start(len - 1)).unwrap();
            f.write_all(&[b[0] ^ 0xFF]).unwrap();
            f.sync_all().unwrap();
        }
        // Only frame 1 (the un-corrupted prefix) replays.
        assert_eq!(replay_seqs(&path, 0), vec![1]);
    }

    #[test]
    fn replay_missing_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.aof");
        assert_eq!(replay_seqs(&path, 0), Vec::<u64>::new());
    }
}

// ---------------------------------------------------------------------------
// THE CRUX: RDB + AOF recovery WITHOUT broker replay (Stage 2 Phase 2f-3).
//
// The whole durability story end-to-end, with the log out of the picture:
//
//   1. A "live" engine applies ops 1..=A via `apply_raft_entry`, with every op
//      ALSO appended to an AOF.
//   2. At seq S (< A) the segment checkpoint is taken (`flush_to_segments`) and
//      the AOF is `truncate_through(S)`d — so on disk the RDB covers 1..=S and
//      the AOF covers S+1..=A.
//   3. "Restart": a FRESH engine reopens the segment dir (recovers to S), then
//      `replay_aof_into` replays S+1..=A (recovers to A).
//
// The restarted engine's query results — result sets, byte-identical f32 scores,
// retrieved field values, and ordered kNN — must equal the live engine at A. If
// the frame crc/len decode, the seq-skip boundary, or `truncate_through` is
// wrong, recovery diverges (or the torn-tail path panics) and this test fails.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod crux_recovery_tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::storage::Engine;
    use crate::types::{
        Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem,
        IndexRequest, KnnQuery, MatchOp, MatchQuery, QueryNode, RangeBound, RangeQuery,
        SearchRequest, TermQuery, TermsQuery, VectorBackend, VectorMetric,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;

    const DIM: usize = 4;

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    fn vec_fieldspec() -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Vector,
            analyzer: None,
            multi: None,
            dim: Some(DIM as u32),
            metric: Some(VectorMetric::L2),
            backend: Some(VectorBackend::FlatCpu),
            quantize: None,
        }
    }

    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("num".into(), fieldspec(FieldType::Number, None));
        fields.insert("kw".into(), fieldspec(FieldType::Keyword, None));
        fields.insert("tags".into(), fieldspec(FieldType::Set, None));
        fields.insert(
            "body".into(),
            fieldspec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
        );
        fields.insert("sig".into(), fieldspec(FieldType::Hash, None));
        fields.insert("emb".into(), vec_fieldspec());
        CreateCollectionRequest { fields }
    }

    /// Build an `Index` entry for one doc across all six fields.
    fn index_entry(
        coll: &str,
        eid: &str,
        n: f64,
        kw: &str,
        tag: &str,
        tok: bool,
        sig: u64,
        emb: &[f32],
    ) -> RaftLogEntry {
        RaftLogEntry::Index {
            collection_id: coll.into(),
            req: IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: eid.into(),
                        field: "num".into(),
                        value: FieldValue::Number(n),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "kw".into(),
                        value: FieldValue::String(kw.into()),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "tags".into(),
                        value: FieldValue::StringList(vec![tag.into()]),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "body".into(),
                        value: FieldValue::String(if tok {
                            "tok filler".into()
                        } else {
                            "filler".into()
                        }),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "sig".into(),
                        value: FieldValue::String(format!("{sig:016x}")),
                        version: None,
                    },
                    IndexItem {
                        external_id: eid.into(),
                        field: "emb".into(),
                        value: FieldValue::Vector(emb.to_vec()),
                        version: None,
                    },
                ],
                request_id: None,
            },
        }
    }

    fn req(query: QueryNode, limit: u32) -> SearchRequest {
        SearchRequest {
            query,
            limit,
            offset: 0,
            cursor: None,
            routing_key: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, coll: &str, query: QueryNode, limit: u32) -> Vec<(String, u32)> {
        e.search(coll, req(query, limit))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score.to_bits()))
            .collect()
    }

    fn set_of(rows: &[(String, u32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }
    fn scores_of(rows: &[(String, u32)]) -> BTreeMap<String, u32> {
        rows.iter().map(|(e, s)| (e.clone(), *s)).collect()
    }

    fn driven(extra: QueryNode) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            extra,
        ])
    }

    /// Full query battery: predicate legs (range/term/setmem/point/bm25/hamming)
    /// as (set, byte-scores), kNN as the ordered ranked vec, and a doc count.
    fn battery(e: &Engine, coll: &str) -> Vec<(BTreeSet<String>, BTreeMap<String, u32>)> {
        let legs = vec![
            driven(QueryNode::Range(RangeQuery {
                field: "num".into(),
                gt: None,
                gte: Some(RangeBound::Number(2.0)),
                lt: Some(RangeBound::Number(9.0)),
                lte: None,
            })),
            driven(QueryNode::Term(TermQuery {
                field: "kw".into(),
                value: FieldValue::String("a".into()),
            })),
            driven(QueryNode::Terms(TermsQuery {
                field: "tags".into(),
                values: vec![FieldValue::String("red".into())],
            })),
            QueryNode::Term(TermQuery {
                field: "kw".into(),
                value: FieldValue::String("b".into()),
            }),
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            QueryNode::Hamming(crate::types::HammingQuery {
                field: "sig".into(),
                hash: format!("{:016x}", 0u64),
                max_distance: 8,
            }),
        ];
        legs.into_iter()
            .map(|q| {
                let r = run(e, coll, q, 100_000);
                (set_of(&r), scores_of(&r))
            })
            .collect()
    }

    fn knn(e: &Engine, coll: &str, q: &[f32]) -> Vec<(String, u32)> {
        run(
            e,
            coll,
            QueryNode::Knn(KnnQuery {
                field: "emb".into(),
                vector: q.to_vec(),
                k: 8,
            }),
            8,
        )
    }

    /// The full op TRANSCRIPT, in apply order. Two collections, all field types;
    /// base docs (1..=S region) then tail docs (S+1..=A region). Returns the
    /// ordered RaftLogEntry list — applied with seq = index+1.
    fn transcript() -> (Vec<RaftLogEntry>, usize) {
        let mut ops = Vec::new();
        ops.push(RaftLogEntry::CreateCollection {
            collection_id: "alpha".into(),
            req: schema(),
        });
        ops.push(RaftLogEntry::CreateCollection {
            collection_id: "beta".into(),
            req: schema(),
        });
        // Base docs (these end up under the segment checkpoint at S).
        let base = [
            ("d0", 1.0, "a", "red", true, 0u64, [0.1f32, 0.2, 0.3, 0.4]),
            ("d1", 3.0, "b", "blue", true, 3, [0.9, 0.8, 0.7, 0.6]),
            ("d2", 5.0, "a", "red", false, 7, [0.5, 0.5, 0.5, 0.5]),
            ("d3", 7.0, "c", "green", true, 1, [0.2, 0.4, 0.6, 0.8]),
        ];
        for (eid, n, kw, tag, tok, sig, emb) in base {
            ops.push(index_entry("alpha", eid, n, kw, tag, tok, sig, &emb));
            ops.push(index_entry(
                "beta",
                &format!("b{eid}"),
                n + 1.0,
                kw,
                tag,
                tok,
                sig + 1,
                &emb,
            ));
        }
        // S = number of ops so far (the checkpoint boundary).
        let s = ops.len();
        // Tail docs (these end up only in the AOF, S+1..=A).
        let tail = [
            (
                "d4",
                2.5,
                "b",
                "red",
                true,
                0u64,
                [0.11f32, 0.22, 0.33, 0.44],
            ),
            ("d5", 6.5, "a", "blue", true, 7, [0.6, 0.6, 0.6, 0.6]),
            ("d6", 8.5, "c", "green", false, 2, [0.3, 0.3, 0.3, 0.3]),
        ];
        for (eid, n, kw, tag, tok, sig, emb) in tail {
            ops.push(index_entry("alpha", eid, n, kw, tag, tok, sig, &emb));
            ops.push(index_entry(
                "beta",
                &format!("b{eid}"),
                n + 1.0,
                kw,
                tag,
                tok,
                sig + 1,
                &emb,
            ));
        }
        (ops, s)
    }

    #[test]
    fn rdb_plus_aof_recovery_matches_live_without_nats() {
        let (ops, s) = transcript();
        let a = ops.len(); // every op applied; A = total.
        let qa = [0.15f32, 0.25, 0.35, 0.45];

        let dir = tempfile::tempdir().unwrap();
        let seg_dir = dir.path().join("segments");
        std::fs::create_dir_all(&seg_dir).unwrap();
        let aof_path = dir.path().join("aof.log");

        // --- LIVE: apply 1..=A, append every op to the AOF, checkpoint at S. ---
        let live = Arc::new(Engine::new());
        let mut aof = AofWriter::open_with_policy(&aof_path, FsyncPolicy::Always).unwrap();
        for (i, op) in ops.iter().enumerate() {
            let seq = (i + 1) as u64;
            live.apply_raft_entry(op.clone()).unwrap();
            aof.append(seq, &WalRecord::new(op.clone())).unwrap();
            if seq == s as u64 {
                // RDB checkpoint at S, then trim the AOF through S — so on disk the
                // segment covers 1..=S and the AOF covers S+1..=A only.
                live.flush_to_segments(&seg_dir, s as u64).unwrap();
                aof.truncate_through(s as u64).unwrap();
            }
        }
        aof.sync().unwrap();

        // On-disk shape sanity: the AOF now holds exactly S+1..=A.
        let mut remaining = Vec::new();
        AofReader::replay(&aof_path, 0, |seq, _| remaining.push(seq)).unwrap();
        assert_eq!(
            remaining,
            ((s as u64 + 1)..=(a as u64)).collect::<Vec<_>>(),
            "AOF must hold exactly the post-checkpoint tail"
        );

        let live_alpha = battery(&live, "alpha");
        let live_beta = battery(&live, "beta");
        let live_knn_alpha = knn(&live, "alpha", &qa);
        let live_knn_beta = knn(&live, "beta", &qa);

        // --- RESTART: fresh engine, RDB reopen → AOF replay (no broker tail). ---
        let restarted = Arc::new(Engine::new());
        let s_recovered = restarted.reopen_from_segment_dir(&seg_dir).unwrap();
        assert_eq!(
            s_recovered, s as u64,
            "RDB must restore to the checkpoint seq S"
        );
        let a_recovered = replay_aof_into(&restarted, &aof_path, s_recovered).unwrap();
        assert_eq!(a_recovered, a as u64, "AOF replay must advance to A");

        // The restarted engine must be byte-identical to the live engine at A.
        assert_eq!(
            battery(&restarted, "alpha"),
            live_alpha,
            "alpha legs diverged after RDB+AOF recovery"
        );
        assert_eq!(
            battery(&restarted, "beta"),
            live_beta,
            "beta legs diverged after RDB+AOF recovery"
        );
        assert_eq!(
            knn(&restarted, "alpha", &qa),
            live_knn_alpha,
            "alpha kNN diverged after RDB+AOF recovery"
        );
        assert_eq!(
            knn(&restarted, "beta", &qa),
            live_knn_beta,
            "beta kNN diverged after RDB+AOF recovery"
        );
        assert_eq!(
            restarted.stats("alpha").unwrap().documents_indexed,
            live.stats("alpha").unwrap().documents_indexed
        );
        assert_eq!(
            restarted.stats("beta").unwrap().documents_indexed,
            live.stats("beta").unwrap().documents_indexed
        );
    }

    /// Recovery is robust to a torn AOF tail: a crash mid-append leaves a partial
    /// frame; recovery replays the good prefix and the engine still converges to
    /// the last DURABLE op (no panic, no divergence on the good prefix).
    #[test]
    fn recovery_tolerates_torn_aof_tail() {
        let (ops, s) = transcript();
        let qa = [0.15f32, 0.25, 0.35, 0.45];
        let dir = tempfile::tempdir().unwrap();
        let seg_dir = dir.path().join("segments");
        std::fs::create_dir_all(&seg_dir).unwrap();
        let aof_path = dir.path().join("aof.log");

        // Apply + append all but the LAST op durably; checkpoint at S.
        let live = Arc::new(Engine::new());
        let mut aof = AofWriter::open_with_policy(&aof_path, FsyncPolicy::Always).unwrap();
        let last = ops.len() - 1;
        for (i, op) in ops.iter().enumerate().take(last) {
            let seq = (i + 1) as u64;
            live.apply_raft_entry(op.clone()).unwrap();
            aof.append(seq, &WalRecord::new(op.clone())).unwrap();
            if seq == s as u64 {
                live.flush_to_segments(&seg_dir, s as u64).unwrap();
                aof.truncate_through(s as u64).unwrap();
            }
        }
        aof.sync().unwrap();
        let good_len = std::fs::metadata(&aof_path).unwrap().len();

        // Simulate a crash mid-append of the FINAL op: a header whose length
        // overruns EOF, plus a stray byte.
        {
            use std::io::Write as _;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&aof_path)
                .unwrap();
            let mut hdr = [0u8; HEADER_LEN];
            hdr[0..8].copy_from_slice(&((ops.len()) as u64).to_le_bytes());
            hdr[8..12].copy_from_slice(&999_999u32.to_le_bytes());
            f.write_all(&hdr).unwrap();
            f.write_all(&[0x42]).unwrap();
            f.sync_all().unwrap();
        }

        // The live oracle: exactly the durable prefix (NOT the torn final op).
        let live_alpha = battery(&live, "alpha");
        let live_beta = battery(&live, "beta");
        let live_knn = knn(&live, "alpha", &qa);

        // Recovery: RDB reopen → AOF replay. The torn tail is skipped cleanly.
        let restarted = Arc::new(Engine::new());
        let s_rec = restarted.reopen_from_segment_dir(&seg_dir).unwrap();
        assert_eq!(s_rec, s as u64);
        let a_rec = replay_aof_into(&restarted, &aof_path, s_rec).unwrap();
        assert_eq!(
            a_rec,
            (ops.len() - 1) as u64,
            "torn final frame must not be replayed"
        );

        assert_eq!(
            battery(&restarted, "alpha"),
            live_alpha,
            "alpha diverged after torn-tail recovery"
        );
        assert_eq!(
            battery(&restarted, "beta"),
            live_beta,
            "beta diverged after torn-tail recovery"
        );
        assert_eq!(
            knn(&restarted, "alpha", &qa),
            live_knn,
            "kNN diverged after torn-tail recovery"
        );

        // And the next writer open truncates the torn tail back to the prefix.
        let _w = AofWriter::open(&aof_path).unwrap();
        assert_eq!(std::fs::metadata(&aof_path).unwrap().len(), good_len);
    }
}
// CODEGEN-END

// Candidate AOF red tests to place in `aof::tests` after the cursor seam lands:
//
// 1. replay_admission_waits_before_mutating_or_advancing:
//    create a budget-limited Engine, retain a checkpointable record charge to
//    fill its budget, append the next create/index frame, and start replay on a
//    thread. Assert the query and capture sequence remain at the prefix while
//    the replay thread blocks. Drive the independent checkpoint, join replay,
//    then assert the new document and its exact sequence are present after cold
//    reopen.
//
// 2. replay_preparation_error_keeps_frame_and_watermark:
//    arrange a record whose admitted prepared-text staging fails through the
//    existing test failure seam. Assert replay returns Err, its sequence stays
//    at the prefix, and a second AOF reader still returns that same frame.
