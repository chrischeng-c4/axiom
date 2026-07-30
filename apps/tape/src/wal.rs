// <HANDWRITE gap="missing-generator:logic:tape-wal" tracker="#3052" reason="Tape's single-node append-only WAL + group-commit commit coordinator: TapeCommand-framed durable log, snapshot+truncate compaction, and torn-tail recovery built on storage-durable's FramedLogWriter/FramedLogReader/SnapshotFileStore primitives.">
//! Append-only, group-commit write-ahead log for `tape serve --data-dir`
//! (WI #3052), replacing the per-request whole-file journal rewrite in
//! [`crate::server::AppState::persist`].
//!
//! # What one frame encodes
//!
//! A frame holds one JSON-encoded [`TapeCommand`], never [`TapeJournal`]
//! state. [`TapeJournal::append_at`] (`apps/tape/src/lib.rs`) calls
//! `enforce_retention` as a side effect of appending, which can *delete*
//! events; logging post-mutation state would silently lose that
//! deleted-event history. Logging the command and replaying it through the
//! shared [`crate::raft::apply_command`] reproduces retention enforcement
//! (and every other side effect) identically on every replay -- the exact
//! same function the Raft-replicated path applies through, so the two
//! cannot drift.
//!
//! # Layout
//!
//! Both paths live directly under the caller's `--data-dir`, with fixed
//! names chosen so neither can collide with the `.storage_full_probe` file
//! the ENOSPC re-probe loop writes there (`apps/tape/src/bin/tape.rs`):
//!
//! - WAL: `<dir>/journal.wal`
//! - Snapshots: `<dir>/journal-<seq>.snap` via [`SnapshotFileStore`]
//!
//! Nothing here scans the directory for arbitrary segment files.
//!
//! # Group commit
//!
//! [`WalStore::commit`] encodes and appends every command in a batch, then
//! performs exactly **one** `fsync` covering the whole batch, and only then
//! takes the journal lock to apply the commands in order. The lock is never
//! held across the fsync. If any append or the sync fails, the batch fails
//! closed: no command in it is applied, and the caller gets the error back
//! with its [`std::io::ErrorKind`] preserved (mirroring
//! `apps/tape/src/server.rs`'s `flatten_atomic_write_error`) so a future
//! caller can distinguish ENOSPC/EIO from an ordinary failure. That single
//! failed batch is not the only thing at risk: a failure can still have
//! landed some of its frames on disk (an `append` cannot be undone), so a
//! later batch reusing the same starting seq would produce a duplicate
//! on-disk frame and replay it twice. `WalStore` closes that window by
//! poisoning itself on any durability failure -- see the `poisoned` field
//! doc comment on the struct -- so every subsequent `commit` fails until the
//! caller reopens from disk.
//!
//! # Recovery
//!
//! [`WalStore::open`] loads the newest snapshot (if any), decodes it into a
//! [`TapeJournal`], then replays every WAL frame after the snapshot's
//! sequence through `apply_command`. [`FramedLogWriter::open`] truncates a
//! torn tail (a partial frame from a crash mid-write) before this module
//! ever reads a byte, and [`FramedLogReader::read_frames`] stops cleanly at
//! the first unreadable frame -- AC5 ("recovers all prior records and drops
//! only the torn one") is a property of using those two calls correctly,
//! not logic this module reimplements.

use std::path::Path;
use std::sync::Mutex;

use storage_durable::{FramedLogReader, FramedLogWriter, FsyncPolicy, SnapshotFileStore};

use crate::raft::{apply_command, TapeCommand, TapeOutcome};
use crate::TapeJournal;

/// Fixed WAL filename under `--data-dir`. Chosen so it cannot collide with
/// `.storage_full_probe` (written by `spawn_storage_full_reprobe` in
/// `apps/tape/src/bin/tape.rs`).
const WAL_FILE_NAME: &str = "journal.wal";

/// Snapshot file prefix/extension under `--data-dir`: `journal-<seq>.snap`.
const SNAPSHOT_PREFIX: &str = "journal";
const SNAPSHOT_EXTENSION: &str = "snap";

/// How many committed frames accumulate before [`WalStore::commit`] drives a
/// snapshot + WAL truncate. Mirrors the shape of `raft::SNAPSHOT_EVERY`.
/// [`WalStore::open_with_snapshot_threshold`] is the real configuration seam
/// step 3 wires up (e.g. from a CLI flag or env var); the unit tests below
/// are just its first consumer, using a small value so they don't need to
/// drive a thousand real fsyncs to exercise snapshot + truncate.
pub const DEFAULT_SNAPSHOT_THRESHOLD: u64 = 1024;

/// Single-node durable commit coordinator for one `--data-dir`'s journal.
///
/// Holds the open WAL writer and the snapshot store; does not hold the
/// [`TapeJournal`] itself -- callers pass the shared `Arc<Mutex<TapeJournal>>`
/// (or any `&Mutex<TapeJournal>`) into [`Self::commit`] each time, matching
/// how `TapeStateMachine` already shares one journal across call sites.
pub struct WalStore {
    wal: FramedLogWriter,
    snapshots: SnapshotFileStore,
    /// The seq the *next* appended frame will use.
    next_seq: u64,
    /// Committed frames since the last successful snapshot + truncate.
    frames_since_snapshot: u64,
    snapshot_threshold: u64,
    /// Set for the whole duration of a `commit`'s durable-write region (every
    /// `append` through `sync`), and cleared only once that region completes
    /// successfully. A durability failure anywhere in that region -- an
    /// `append` or the covering `sync` returning `Err` -- leaves this `true`
    /// and poisons the store: every subsequent `commit` fails immediately
    /// until the caller reopens from disk.
    ///
    /// This is not defensive extra caution; it is the fix for a real bug.
    /// `append`'s effects are not undoable once written (a partially
    /// appended batch cannot be "rolled back" out of the file), so a batch
    /// that fails mid-write may still have landed some or all of its frames
    /// on disk even though the batch was never acknowledged. If the *next*
    /// `commit` were allowed to proceed, it would reuse the same starting
    /// `next_seq` (never advanced because the failed batch's `?` returned
    /// before `next_seq` was updated), producing a second on-disk frame with
    /// the same seq as an already-landed one. `FramedLogReader::read_frames`
    /// filters by `seq > from_seq` only -- it does not deduplicate -- so a
    /// later replay would apply *both* frames: a duplicate append, or a
    /// duplicate ack. Poisoning the whole store closes that window instead
    /// of patching each failure site individually. Step 3 wires this state
    /// to the existing `TapeMetrics::mark_storage_degraded` sticky
    /// read-only/507 path.
    poisoned: bool,
    #[cfg(test)]
    fail_next_sync: std::sync::atomic::AtomicBool,
}

impl WalStore {
    /// Open (or create) the WAL + snapshot store under `dir`, recovering a
    /// [`TapeJournal`] by replaying the newest snapshot plus every WAL frame
    /// after it. Returns the store positioned to append after the last
    /// replayed frame, and the recovered journal.
    pub fn open(dir: impl AsRef<Path>) -> std::io::Result<(WalStore, TapeJournal)> {
        Self::open_with_snapshot_threshold(dir, DEFAULT_SNAPSHOT_THRESHOLD)
    }

    /// Same as [`Self::open`] with an explicit snapshot-trigger threshold --
    /// the seam a caller (step 3) configures the snapshot cadence through.
    pub fn open_with_snapshot_threshold(
        dir: impl AsRef<Path>,
        snapshot_threshold: u64,
    ) -> std::io::Result<(WalStore, TapeJournal)> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let snapshots = SnapshotFileStore::new(
            dir,
            SNAPSHOT_PREFIX,
            SNAPSHOT_EXTENSION,
            FsyncPolicy::Always,
        )
        .map_err(flatten_io_error)?;
        // `load_latest` doesn't hand back the winning `seq`, which recovery
        // needs to bound the WAL replay -- so this reads the sorted listing
        // itself instead.
        let snapshot_files = snapshots.snapshots().map_err(flatten_io_error)?;
        let (mut journal, snapshot_seq) = match snapshot_files.last() {
            Some(latest) => {
                let bytes = std::fs::read(&latest.path)?;
                let journal: TapeJournal = serde_json::from_slice(&bytes).map_err(json_err)?;
                (journal, latest.seq)
            }
            None => (TapeJournal::default(), 0),
        };

        let wal_path = dir.join(WAL_FILE_NAME);
        // Opening the writer truncates a torn tail as a side effect (scans to
        // the last good frame boundary and `set_len`s past it) before we read
        // anything below -- see the module doc comment.
        let wal = FramedLogWriter::open(&wal_path, FsyncPolicy::Os).map_err(flatten_io_error)?;

        let frames =
            FramedLogReader::read_frames(&wal_path, snapshot_seq).map_err(flatten_io_error)?;
        let mut next_seq = snapshot_seq + 1;
        let mut frames_since_snapshot = 0u64;
        for frame in frames {
            let command: TapeCommand = serde_json::from_slice(&frame.payload).map_err(json_err)?;
            apply_command(&mut journal, command);
            next_seq = frame.seq + 1;
            frames_since_snapshot += 1;
        }

        Ok((
            WalStore {
                wal,
                snapshots,
                next_seq,
                frames_since_snapshot,
                snapshot_threshold,
                poisoned: false,
                #[cfg(test)]
                fail_next_sync: std::sync::atomic::AtomicBool::new(false),
            },
            journal,
        ))
    }

    /// Group-commit one batch of pending commands: encode + append every
    /// command, one fsync barrier over the whole batch, then apply them in
    /// order under a single lock acquisition. Fails closed -- if any append
    /// or the sync errors, this returns `Err` and not one command in
    /// `commands` has been applied to `journal`. A durability failure (as
    /// opposed to the store already being poisoned from a prior one) also
    /// poisons the store for every subsequent call -- see the `poisoned`
    /// field doc comment for why that is required, not optional caution.
    pub fn commit(
        &mut self,
        commands: Vec<TapeCommand>,
        journal: &Mutex<TapeJournal>,
    ) -> std::io::Result<Vec<TapeOutcome>> {
        if commands.is_empty() {
            return Ok(Vec::new());
        }

        if self.poisoned {
            return Err(std::io::Error::other(
                "wal store poisoned by an earlier durability failure; \
                 no further commits until reopen",
            ));
        }

        // Set BEFORE entering the durable-write region so every early return
        // below (`?`, or the test-only injected sync failure) leaves the
        // store poisoned by construction -- there is no failure site that
        // has to remember to poison it by hand.
        self.poisoned = true;

        let base_seq = self.next_seq;
        for (i, command) in commands.iter().enumerate() {
            let seq = base_seq + i as u64;
            let payload = serde_json::to_vec(command).map_err(json_err)?;
            self.wal.append(seq, &payload).map_err(flatten_io_error)?;
        }

        #[cfg(test)]
        if self
            .fail_next_sync
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            // Injected right where the real `sync()` call below would fail:
            // every frame in this batch has already been appended (as a real
            // `sync` failure would leave it), but nothing has been synced or
            // applied yet.
            return Err(std::io::Error::other("injected sync failure (test)"));
        }

        // The single group-commit barrier: one fsync covers every frame just
        // appended above. The journal lock is not held here or above -- it is
        // only taken in the apply loop below, after this line has already
        // returned `Ok`.
        self.wal.sync().map_err(flatten_io_error)?;
        self.next_seq = base_seq + commands.len() as u64;
        // Only a fully successful durable-write region un-poisons the store.
        self.poisoned = false;

        // WAL-order-equals-apply-order: this loop walks `commands` -- the
        // exact same `Vec`, in the exact same order -- that the append loop
        // above just walked and synced. Frame `base_seq + i` on disk and the
        // i-th outcome applied here always correspond to the same command,
        // because nothing reorders `commands` between the two loops.
        let mut outcomes = Vec::with_capacity(commands.len());
        {
            let mut journal = journal.lock().expect("journal mutex poisoned");
            for command in commands {
                outcomes.push(apply_command(&mut journal, command));
            }
        }

        self.frames_since_snapshot += outcomes.len() as u64;
        if self.frames_since_snapshot >= self.snapshot_threshold {
            let last_seq = self.next_seq - 1;
            // Serializing the snapshot is inside the swallowed region for the
            // same reason `snapshot_and_truncate` itself is: the batch is
            // already synced and applied by now, so an encode failure here
            // must not be reported as a failed commit. A `?` on this line
            // would hand step 3 an `Err` for a mutation that in fact
            // succeeded -- and step 3 maps `Err` to 507 + sticky degraded,
            // which is exactly the shape that invites a client retry and a
            // duplicate append.
            let snapshot_result = {
                let journal = journal.lock().expect("journal mutex poisoned");
                serde_json::to_vec(&*journal).map_err(json_err)
            }
            .and_then(|bytes| self.snapshot_and_truncate(last_seq, &bytes));
            match snapshot_result {
                Ok(()) => self.frames_since_snapshot = 0,
                Err(error) => {
                    // The batch above is already durably committed and
                    // applied; a snapshot/truncate hiccup only means the WAL
                    // keeps growing until the next successful attempt, not
                    // that this commit failed.
                    tracing::warn!(
                        %error,
                        "wal: snapshot+truncate failed; WAL will keep growing until the next successful attempt (the batch itself is committed)"
                    );
                }
            }
        }

        Ok(outcomes)
    }

    /// Save a snapshot at `seq`, truncate the WAL through it, and keep only
    /// the newest snapshot file.
    ///
    /// This is deliberately a bare `serde_json::to_vec(&TapeJournal)`, NOT
    /// the same bytes as `GET /admin/backup` / `raft::snapshot_bytes`, which
    /// serialize `raft::JournalSnapshot { up_to, journal, completed_proposals
    /// }`. The two formats are intentionally different: this snapshot is a
    /// purely internal recovery artifact for `WalStore::open`'s own replay,
    /// with no raft applied-index or proposal-dedupe concerns, and it is
    /// never read by anything outside this module. The #3052 out-of-scope
    /// boundary ("snapshot/backup wire format is unchanged") and AC6 ("`GET
    /// /admin/backup` is byte-identical to the old path") both live entirely
    /// on the `raft::snapshot_bytes` / `/admin/backup` side, which this
    /// function never touches.
    ///
    /// A failure here does NOT poison the store the way [`Self::commit`]'s
    /// durable-write region does: by the time this runs, the batch that
    /// triggered it is already durably synced AND applied to `journal`. A
    /// snapshot/truncate failure only means the WAL keeps growing instead of
    /// being compacted -- it is a maintenance hiccup, not an unresolved
    /// durability gap, so `commit` logs and continues rather than poisoning.
    fn snapshot_and_truncate(&mut self, seq: u64, snapshot_bytes: &[u8]) -> std::io::Result<()> {
        self.snapshots
            .save(seq, snapshot_bytes)
            .map_err(flatten_io_error)?;
        self.wal.truncate_through(seq).map_err(flatten_io_error)?;
        self.snapshots.prune(1).map_err(flatten_io_error)?;
        Ok(())
    }

    /// Test-only fault injection: the next [`Self::commit`] call appends its
    /// frames normally (so they can land on disk unsynced, matching a real
    /// crash-during-sync) and then fails exactly where the real `sync()`
    /// call would run, before anything is applied to the journal (mirrors
    /// `AppState::inject_storage_full` in `apps/tape/src/server.rs`).
    #[cfg(test)]
    fn inject_next_sync_failure(&self) {
        self.fail_next_sync
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Collapse an `anyhow::Error` from a `storage_durable` call back into a
/// `std::io::Error` without losing its [`std::io::ErrorKind`], the same
/// discipline `apps/tape/src/server.rs`'s `flatten_atomic_write_error` uses
/// for exactly this reason: a caller (WI #3052 step 3) needs to discriminate
/// ENOSPC/EIO from an ordinary failure, which a bare `anyhow` chain loses.
fn flatten_io_error(error: anyhow::Error) -> std::io::Error {
    match error.downcast_ref::<std::io::Error>() {
        Some(source) => std::io::Error::new(source.kind(), format!("{error:#}")),
        None => std::io::Error::other(format!("{error:#}")),
    }
}

/// A `serde_json` encode/decode failure is corruption or a programmer error,
/// never a durability signal -- map it to `InvalidData` rather than `Other`
/// so it is at least distinguishable from an I/O failure.
fn json_err(error: serde_json::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RetentionPolicy;

    fn append_cmd(topic: &str, n: u64, applied_at_ms: u64) -> TapeCommand {
        TapeCommand::Append {
            topic: topic.to_string(),
            key: None,
            payload: serde_json::json!({ "n": n }),
            timestamp_ms: applied_at_ms,
            applied_at_ms,
        }
    }

    #[test]
    fn round_trip_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let (mut store, journal) = WalStore::open(dir.path()).unwrap();
        let journal = Mutex::new(journal);

        let outcomes = store
            .commit(
                vec![
                    append_cmd("orders", 1, 100),
                    append_cmd("orders", 2, 100),
                    append_cmd("orders", 3, 100),
                ],
                &journal,
            )
            .unwrap();
        assert_eq!(outcomes.len(), 3);
        let before = journal.lock().unwrap().clone();
        assert_eq!(before.end_offset("orders"), 3);
        drop(store);

        let (store2, recovered) = WalStore::open(dir.path()).unwrap();
        drop(store2);
        assert_eq!(recovered, before);
    }

    #[test]
    fn torn_tail_recovers_every_complete_frame_and_drops_only_the_torn_one() {
        let dir = tempfile::tempdir().unwrap();
        let (mut store, journal) = WalStore::open(dir.path()).unwrap();
        let journal_lock = Mutex::new(journal);
        store
            .commit(
                vec![
                    append_cmd("orders", 1, 100),
                    append_cmd("orders", 2, 100),
                    append_cmd("orders", 3, 100),
                ],
                &journal_lock,
            )
            .unwrap();
        let complete_journal = journal_lock.into_inner().unwrap();
        drop(store);

        // Simulate a crash mid-write of a fourth frame: append a few stray
        // bytes past every complete, already-synced frame -- too short to be
        // a valid frame header, so `scan_good_end` must stop exactly at the
        // boundary of the last complete frame and truncate only this torn
        // tail, not any of the three good frames above it.
        let wal_path = dir.path().join(WAL_FILE_NAME);
        {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&wal_path)
                .unwrap();
            file.write_all(b"\x00\x01\x02").unwrap();
            file.sync_all().unwrap();
        }

        let (store2, recovered) = WalStore::open(dir.path()).unwrap();
        drop(store2);
        // Every complete record survives; nothing beyond the good boundary
        // was fabricated.
        assert_eq!(recovered, complete_journal);
        assert_eq!(recovered.end_offset("orders"), 3);
    }

    #[test]
    fn snapshot_and_truncate_shrinks_wal_and_reopen_still_reconstructs() {
        let dir = tempfile::tempdir().unwrap();
        let (mut store, journal) = WalStore::open_with_snapshot_threshold(dir.path(), 3).unwrap();
        let journal_lock = Mutex::new(journal);

        for n in 0..6u64 {
            store
                .commit(vec![append_cmd("orders", n, 100)], &journal_lock)
                .unwrap();
        }
        let complete_journal = journal_lock.into_inner().unwrap();
        drop(store);

        let wal_path = dir.path().join(WAL_FILE_NAME);
        let wal_len_after_snapshot = std::fs::metadata(&wal_path).unwrap().len();
        // Six single-command commits with a threshold of 3 crosses the
        // threshold twice; the WAL must never grow to hold all six frames.
        assert!(wal_len_after_snapshot < 6 * 64);

        let (store2, recovered) = WalStore::open_with_snapshot_threshold(dir.path(), 3).unwrap();
        drop(store2);
        assert_eq!(recovered, complete_journal);
        assert_eq!(recovered.end_offset("orders"), 6);
    }

    #[test]
    fn retention_pruning_replays_the_pruned_journal_not_the_pre_pruned_one() {
        let dir = tempfile::tempdir().unwrap();
        let (mut store, journal) = WalStore::open(dir.path()).unwrap();
        let journal_lock = Mutex::new(journal);

        let mut commands: Vec<TapeCommand> = (0..5).map(|n| append_cmd("orders", n, 100)).collect();
        commands.push(TapeCommand::RetentionPut {
            topic: "orders".to_string(),
            policy: RetentionPolicy {
                min_offset: Some(3),
                max_age_seconds: None,
                protected_consumers: Vec::new(),
            },
            now_ms: 100,
        });
        store.commit(commands, &journal_lock).unwrap();

        let pruned = journal_lock.into_inner().unwrap();
        // enforce_retention ran as a side effect of the RetentionPut command
        // (and of every prior append): only offsets >= 3 remain resident,
        // even though 5 events were appended.
        assert_eq!(pruned.replay("orders", None, None, None).len(), 2);
        assert_eq!(pruned.end_offset("orders"), 5);
        drop(store);

        let (store2, recovered) = WalStore::open(dir.path()).unwrap();
        drop(store2);
        // Replaying commands (not post-state) reproduces the SAME pruned
        // journal -- this is the property that requires logging TapeCommand
        // rather than TapeJournal state.
        assert_eq!(recovered, pruned);
        assert_eq!(recovered.replay("orders", None, None, None).len(), 2);
    }

    /// A sync failure after frames have already landed on disk must: (1)
    /// leave the in-memory journal completely untouched; (2) poison the
    /// store so the orphaned batch's seq is never reused by a later commit
    /// (the actual bug this pins down: a reused seq would duplicate a frame,
    /// and `FramedLogReader::read_frames` does not deduplicate by seq); and
    /// (3) on reopen, replay the orphaned batch AT MOST ONCE. An
    /// unacknowledged batch may or may not survive a crash right at the sync
    /// boundary -- the caller already received an error, so either outcome
    /// is a legitimate answer to "did it happen" -- but it must never come
    /// back twice, which is the property poisoning exists to guarantee.
    #[test]
    fn sync_failure_poisons_the_store_and_the_orphaned_batch_replays_at_most_once() {
        let dir = tempfile::tempdir().unwrap();
        let (mut store, journal) = WalStore::open(dir.path()).unwrap();
        let journal_lock = Mutex::new(journal);
        store.inject_next_sync_failure();

        // The append loop runs normally; only the sync (injected) fails.
        let result = store.commit(vec![append_cmd("orders", 1, 100)], &journal_lock);
        assert!(result.is_err());
        // (1) journal untouched -- apply never runs before a successful sync.
        assert_eq!(journal_lock.lock().unwrap().end_offset("orders"), 0);

        // (2) poisoned: a later commit must not be allowed to reuse the same
        // starting seq the failed batch already wrote frames at.
        let retry = store.commit(vec![append_cmd("orders", 2, 100)], &journal_lock);
        assert!(retry.is_err());
        assert_eq!(journal_lock.lock().unwrap().end_offset("orders"), 0);
        drop(store);

        // (3) reopen replays whatever of the orphaned batch actually landed
        // on disk exactly once -- not zero-or-two times. `FramedLogWriter`'s
        // buffered writer flushes its already-appended-but-unsynced bytes on
        // drop, so in this test the frame lands and IS replayed; the
        // property under test is that it is never replayed twice.
        let (store2, recovered) = WalStore::open(dir.path()).unwrap();
        drop(store2);
        assert_eq!(recovered.replay("orders", None, None, None).len(), 1);
        assert_eq!(recovered.end_offset("orders"), 1);
    }

    #[test]
    fn empty_wal_no_snapshot_opens_into_an_empty_journal() {
        let dir = tempfile::tempdir().unwrap();
        let (store, journal) = WalStore::open(dir.path()).unwrap();
        drop(store);
        assert_eq!(journal, TapeJournal::default());
    }
}
// </HANDWRITE>
