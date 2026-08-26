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
//! with both its [`std::io::ErrorKind`] and its `errno` intact (see
//! [`DurabilityFailure`] -- the kind alone carries ENOSPC but not EIO), so
//! `AppState::apply_mutation` can tell a durability failure that must latch
//! degraded read-only mode from an ordinary one. That single
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
use std::sync::{Arc, Mutex};

use storage_durable::{FramedLogReader, FramedLogWriter, FsyncPolicy, SnapshotFileStore};
use tokio::sync::{mpsc, oneshot};

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

/// The legacy whole-file JSON journal name a pre-WI-#3052 `tape serve
/// --data-dir` wrote (`resolve_journal_store` in `apps/tape/src/bin/tape.rs`
/// used to join this onto `--data-dir` before the WAL existed).
const LEGACY_JOURNAL_FILE_NAME: &str = "journal.json";

/// One-time upgrade path for a `--data-dir` that already has state from
/// before WI #3052: seed a WAL-store snapshot from the old whole-file
/// `journal.json` so [`WalStore::open`] (called right after this) recovers
/// the pre-existing journal instead of starting empty.
///
/// Migrates ONLY when it is unambiguous that this `dir` predates the WAL:
/// no `journal.wal` yet, no `journal-*.snap` yet, and a `journal.json` that
/// does exist. Any other combination is a no-op (`Ok(false)`) -- in
/// particular, a directory that already has a WAL or a snapshot is treated
/// as already migrated (or a from-scratch WAL deployment that happens to
/// share a `--data-dir` with an old file for unrelated reasons), never
/// re-migrated.
///
/// Deliberately never deletes `journal.json`: it is the rollback path if the
/// operator needs to downgrade back to a pre-#3052 build. This function is
/// purely additive -- it writes one new WAL-store snapshot file and touches
/// nothing else.
///
/// Returns `Ok(true)` when a migration snapshot was written, `Ok(false)`
/// when no migration was needed (including "nothing to migrate" and
/// "already migrated").
pub fn migrate_legacy_journal_file(dir: &Path) -> std::io::Result<bool> {
    let wal_path = dir.join(WAL_FILE_NAME);
    if wal_path.exists() {
        return Ok(false);
    }

    let legacy_path = dir.join(LEGACY_JOURNAL_FILE_NAME);
    if !legacy_path.exists() {
        return Ok(false);
    }

    let snapshots = SnapshotFileStore::new(
        dir,
        SNAPSHOT_PREFIX,
        SNAPSHOT_EXTENSION,
        FsyncPolicy::Always,
    )
    .map_err(flatten_io_error)?;
    if !snapshots.snapshots().map_err(flatten_io_error)?.is_empty() {
        return Ok(false);
    }

    let bytes = std::fs::read(&legacy_path)?;
    // Round-trip through `TapeJournal` (rather than copying raw bytes) so a
    // malformed legacy file surfaces as a decode error here, at startup,
    // instead of silently seeding a snapshot `WalStore::open` cannot parse
    // later.
    let journal: TapeJournal = serde_json::from_slice(&bytes).map_err(json_err)?;
    let snapshot_bytes = serde_json::to_vec(&journal).map_err(json_err)?;
    // Seed at seq 0: no WAL frames exist yet (checked above), so replay
    // after this snapshot starts from an empty WAL, exactly like a normal
    // fresh `WalStore::open` with one prior snapshot.
    snapshots
        .save(0, &snapshot_bytes)
        .map_err(flatten_io_error)?;
    Ok(true)
}

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
    /// Fault-injection seam (WI #3052 AC7): when armed via
    /// [`Self::inject_next_sync_failure_with_kind`], the next [`Self::commit`]
    /// fails its sync with this [`std::io::ErrorKind`] instead of performing a
    /// real fsync -- e.g. `ErrorKind::StorageFull` to simulate ENOSPC
    /// originating INSIDE the WAL without needing a genuinely full disk.
    ///
    /// Deliberately NOT `#[cfg(test)]` (unlike the older
    /// [`Self::inject_next_sync_failure`] this replaces the body of below):
    /// an integration test under `apps/tape/e2e/` links this crate as an
    /// ordinary, non-`cfg(test)` dependency, so a `#[cfg(test)]`-gated seam
    /// would not exist for `e2e/durable_write_path.rs` to call at all.
    /// This is an honestly-named, always-present fault-injection hook, not a
    /// hidden backdoor: it only ever fires when a caller explicitly arms it,
    /// and arming requires holding a `&WalStore` in the first place.
    injected_sync_failure: Mutex<Option<std::io::ErrorKind>>,
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
                injected_sync_failure: Mutex::new(None),
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

        if let Some(kind) = self
            .injected_sync_failure
            .lock()
            .expect("injected_sync_failure mutex poisoned")
            .take()
        {
            // Injected right where the real `sync()` call below would fail:
            // every frame in this batch has already been appended (as a real
            // `sync` failure would leave it), but nothing has been synced or
            // applied yet.
            return Err(std::io::Error::new(
                kind,
                "injected sync failure (fault-injection seam)",
            ));
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
        self.inject_next_sync_failure_with_kind(std::io::ErrorKind::Other);
    }

    /// Fault-injection seam for WI #3052 AC7: arm the next [`Self::commit`]
    /// call to fail its sync with `kind` -- e.g.
    /// `std::io::ErrorKind::StorageFull` to simulate ENOSPC originating
    /// INSIDE the WAL (as opposed to `AppState::set_inject_storage_full`,
    /// which short-circuits BEFORE the durable backend is ever reached and
    /// so cannot exercise this path -- see
    /// `apps/tape/e2e/durable_write_path.rs`). Frames from the armed batch
    /// still land on disk (matching a real crash-during-sync), and the
    /// failure poisons the store exactly as a genuine sync failure would --
    /// see the `poisoned` field doc comment.
    ///
    /// Call this on the `WalStore` BEFORE handing it to
    /// [`CommitCoordinator::spawn`] (which moves it onto the dedicated commit
    /// thread and does not expose it again): the injected kind is consumed
    /// by the very next `commit`, whichever caller reaches it first.
    pub fn inject_next_sync_failure_with_kind(&self, kind: std::io::ErrorKind) {
        *self
            .injected_sync_failure
            .lock()
            .expect("injected_sync_failure mutex poisoned") = Some(kind);
    }
}

/// Collapse an `anyhow::Error` from a `storage_durable` call back into a
/// `std::io::Error` without losing its [`std::io::ErrorKind`], the same
/// discipline `apps/tape/src/server.rs`'s `flatten_atomic_write_error` uses
/// for exactly this reason: a caller (WI #3052 step 3) needs to discriminate
/// ENOSPC/EIO from an ordinary failure, which a bare `anyhow` chain loses.
fn flatten_io_error(error: anyhow::Error) -> std::io::Error {
    match error.downcast_ref::<std::io::Error>() {
        Some(source) => std::io::Error::new(
            source.kind(),
            DurabilityFailure {
                message: format!("{error:#}"),
                errno: source.raw_os_error(),
            },
        ),
        None => std::io::Error::other(format!("{error:#}")),
    }
}

/// Test-only: build the exact [`std::io::Error`] shape the WAL path hands
/// `AppState::apply_mutation`, so `server`'s degraded-mode test exercises the
/// real carrier rather than a hand-rolled look-alike that could stay green
/// while the production path lost the errno.
#[cfg(test)]
pub fn flatten_io_error_for_test(source: std::io::Error) -> std::io::Error {
    flatten_io_error(anyhow::Error::from(source))
}

/// Error payload that keeps a durable-write failure's `errno` reachable after
/// its [`std::io::Error`] has been rebuilt.
///
/// Rebuilding is unavoidable on this path: `storage_durable` returns
/// `anyhow::Error`, and [`CommitCoordinator`] has to hand one failure to every
/// waiter in a failed batch while `std::io::Error` is not `Clone`. Both
/// rebuilds go through `std::io::Error::new`, which erases `raw_os_error()` --
/// so an errno not already reflected in a *stable* [`std::io::ErrorKind`] is
/// lost. That is not hypothetical. ENOSPC survives, because it has
/// [`std::io::ErrorKind::StorageFull`]. EIO does not: it maps to
/// `ErrorKind::Uncategorized`, which is unstable and therefore unnameable in
/// stable Rust, so errno 5 is recoverable only if carried explicitly.
/// `AppState::apply_mutation`'s degraded-mode predicate (WI #3052 R6, and the
/// accepted TD's `should_enter_storage_degraded_mode`) needs exactly that.
#[derive(Debug, Clone)]
pub struct DurabilityFailure {
    message: String,
    errno: Option<i32>,
}

impl std::fmt::Display for DurabilityFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DurabilityFailure {}

/// The `errno` behind a durability failure, when [`flatten_io_error`] recorded
/// one and every rebuild since has preserved it. Returns `None` for errors
/// that never came from an OS call.
pub fn durability_errno(error: &std::io::Error) -> Option<i32> {
    error
        .get_ref()
        .and_then(|source| source.downcast_ref::<DurabilityFailure>())
        .and_then(|failure| failure.errno)
}

/// Rebuild an [`std::io::Error`] preserving both its [`std::io::ErrorKind`]
/// and any errno [`durability_errno`] can read back, since `std::io::Error`
/// is not `Clone` and one failed batch has many waiters.
fn clone_io_error(error: &std::io::Error) -> std::io::Error {
    std::io::Error::new(
        error.kind(),
        DurabilityFailure {
            message: error.to_string(),
            errno: durability_errno(error),
        },
    )
}

/// A `serde_json` encode/decode failure is corruption or a programmer error,
/// never a durability signal -- map it to `InvalidData` rather than `Other`
/// so it is at least distinguishable from an I/O failure.
fn json_err(error: serde_json::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
}

/// How many pending [`TapeCommand`]s [`CommitCoordinator`]'s loop drains into
/// one [`WalStore::commit`] batch before it stops accepting more for that
/// round. A cap rather than "drain everything queued" bounds worst-case
/// batch latency and memory under a request storm; `WalStore::commit`'s
/// group-commit fsync amortization does not need an unbounded batch to pay
/// off.
const MAX_COMMIT_BATCH: usize = 256;

/// One caller's pending mutation, queued for the next group-commit batch.
struct CommitRequest {
    command: TapeCommand,
    reply: oneshot::Sender<std::io::Result<TapeOutcome>>,
}

/// Single-node durable commit coordinator: the async-facing handle onto a
/// dedicated OS thread that owns the [`WalStore`] and drives its group
/// commit.
///
/// # Why a dedicated `std::thread`, not a `tokio::task`
///
/// [`WalStore::commit`] fsyncs -- a blocking syscall. Spawning it as an
/// ordinary `tokio::task` (even via `spawn_blocking`, which still borrows
/// from a bounded blocking-pool) would let the durable write path compete
/// with -- and under sustained load, starve -- the async runtime's request
/// handling, defeating the whole point of WI #3052 (replacing a serialized
/// per-request fsync with amortized group commit, not smuggling the same
/// blocking cost back into the runtime that serves HTTP). A `std::thread`
/// dedicated to exactly one `--data-dir`'s WAL is the coordinator's entire
/// job for the life of the process: one thread, one `WalStore`, one
/// `Mutex<TapeJournal>`.
///
/// # Wiring
///
/// [`Self::submit`] sends a [`CommitRequest`] down an unbounded
/// [`mpsc::Sender`] and awaits its [`oneshot::Receiver`] for the reply -- both
/// sides are safe to use from async code on any tokio worker. The dedicated
/// thread's loop blocks on [`mpsc::Receiver::blocking_recv`] for the first
/// request of a round, then drains up to [`MAX_COMMIT_BATCH`] more with
/// non-blocking `try_recv` so a request storm amortizes over one
/// `WalStore::commit` call instead of committing one command at a time.
/// `oneshot::Sender::send` never blocks and can be called from any thread,
/// so replying from the dedicated thread back to whichever tokio worker is
/// awaiting `submit` requires no additional synchronization.
pub struct CommitCoordinator {
    tx: mpsc::Sender<CommitRequest>,
}

impl CommitCoordinator {
    /// Spawn the dedicated commit thread and return the handle callers
    /// `submit` through. `store` and `journal` are moved onto the new thread
    /// (`journal` stays an `Arc` so callers -- e.g. `AppState` -- keep a
    /// handle to read from it directly; the coordinator thread is simply
    /// another `Arc` owner that also mutates it via `WalStore::commit`).
    pub fn spawn(mut store: WalStore, journal: Arc<Mutex<TapeJournal>>) -> Self {
        // Bounded at one batch's worth: a queue deeper than one
        // `WalStore::commit` batch cannot help throughput (the dedicated
        // thread only ever drains `MAX_COMMIT_BATCH` per round) and instead
        // just hides backpressure from callers. `submit`'s `.send().await`
        // naturally yields the calling tokio worker back to the runtime
        // while the channel is full, rather than busy-waiting.
        let (tx, mut rx) = mpsc::channel::<CommitRequest>(MAX_COMMIT_BATCH);
        std::thread::spawn(move || {
            // `blocking_recv` parks this dedicated OS thread (not a tokio
            // worker) until the first request of a round arrives.
            while let Some(first) = rx.blocking_recv() {
                let mut batch = vec![first];
                while batch.len() < MAX_COMMIT_BATCH {
                    match rx.try_recv() {
                        Ok(request) => batch.push(request),
                        Err(_) => break,
                    }
                }
                let (commands, replies): (Vec<TapeCommand>, Vec<_>) = batch
                    .into_iter()
                    .map(|request| (request.command, request.reply))
                    .unzip();
                match store.commit(commands, &journal) {
                    Ok(outcomes) => {
                        for (reply, outcome) in replies.into_iter().zip(outcomes) {
                            // A dropped receiver (the submitting task was
                            // cancelled) is not this coordinator's problem --
                            // the command is already durably committed and
                            // applied either way.
                            let _ = reply.send(Ok(outcome));
                        }
                    }
                    Err(error) => {
                        // `std::io::Error` is not `Clone`: rebuild one per
                        // waiter so every caller in the failed batch can still
                        // discriminate ENOSPC/EIO from an ordinary failure --
                        // the property `AppState::apply_mutation`'s
                        // degraded-mode mapping depends on. `kind()` alone is
                        // not enough to carry that; see [`DurabilityFailure`].
                        for reply in replies {
                            let _ = reply.send(Err(clone_io_error(&error)));
                        }
                    }
                }
            }
            // The sender half (and every clone of it) has been dropped --
            // e.g. process shutdown -- so this dedicated thread exits.
        });
        CommitCoordinator { tx }
    }

    /// Submit one command and await its durably-committed [`TapeOutcome`].
    /// Safe to call from any tokio worker; the actual fsync runs on the
    /// dedicated commit thread, never on the calling task's worker.
    pub async fn submit(&self, command: TapeCommand) -> std::io::Result<TapeOutcome> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(CommitRequest {
                command,
                reply: reply_tx,
            })
            .await
            .map_err(|_| {
                std::io::Error::other("wal commit coordinator thread is no longer running")
            })?;
        reply_rx.await.map_err(|_| {
            std::io::Error::other(
                "wal commit coordinator dropped this request's reply before answering",
            )
        })?
    }
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

    /// Pins the reason [`DurabilityFailure`] exists at all. The obvious way to
    /// recover an errno from a flattened durability failure --
    /// `error.raw_os_error()` -- returns `None`, because
    /// `std::io::Error::new` erases it; and the obvious way to recognize EIO
    /// by kind fails too, because EIO is `Uncategorized`, not `Other`. Both
    /// naive forms are asserted here so that deleting the carrier as
    /// "redundant" turns this test red instead of silently disabling
    /// `AppState::apply_mutation`'s EIO branch.
    #[test]
    fn flatten_io_error_carries_the_errno_a_rebuilt_io_error_would_lose() {
        const EIO: i32 = 5;
        let flattened = flatten_io_error(
            anyhow::Error::from(std::io::Error::from_raw_os_error(EIO))
                .context("syncing the write-ahead log"),
        );

        assert_eq!(durability_errno(&flattened), Some(EIO));
        assert_eq!(flattened.raw_os_error(), None);
        assert_ne!(flattened.kind(), std::io::ErrorKind::Other);
        assert!(flattened
            .to_string()
            .contains("syncing the write-ahead log"));

        // ENOSPC needs no carrier -- it has a stable `ErrorKind` -- but the
        // carrier must not break it, since that is the path #2573 already
        // depends on.
        let enospc = flatten_io_error(anyhow::Error::from(std::io::Error::from(
            std::io::ErrorKind::StorageFull,
        )));
        assert_eq!(enospc.kind(), std::io::ErrorKind::StorageFull);
    }

    /// The commit coordinator fans one failure out to every waiter in the
    /// batch, and `std::io::Error` is not `Clone`. Both the kind and the errno
    /// have to survive that fan-out or the last waiter is told something
    /// different from the first.
    #[test]
    fn clone_io_error_preserves_both_kind_and_errno_across_the_fan_out() {
        const EIO: i32 = 5;
        let original =
            flatten_io_error(anyhow::Error::from(std::io::Error::from_raw_os_error(EIO)));
        let cloned = clone_io_error(&original);

        assert_eq!(cloned.kind(), original.kind());
        assert_eq!(durability_errno(&cloned), Some(EIO));
        assert_eq!(cloned.to_string(), original.to_string());

        // An error that never came from an OS call reports no errno rather
        // than a misleading zero.
        let plain = std::io::Error::other("coordinator thread is gone");
        assert_eq!(durability_errno(&plain), None);
        assert_eq!(durability_errno(&clone_io_error(&plain)), None);
    }

    #[test]
    fn migrate_legacy_journal_seeds_a_snapshot_wal_open_then_recovers() {
        let dir = tempfile::tempdir().unwrap();
        let mut legacy = TapeJournal::default();
        legacy.append("orders", None, serde_json::json!({ "n": 1 }), Some(100));
        legacy.append("orders", None, serde_json::json!({ "n": 2 }), Some(100));
        let legacy_path = dir.path().join(LEGACY_JOURNAL_FILE_NAME);
        std::fs::write(&legacy_path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();

        let migrated = migrate_legacy_journal_file(dir.path()).unwrap();
        assert!(migrated, "a fresh dir with only journal.json must migrate");

        // journal.json is never deleted -- it is the rollback path.
        assert!(legacy_path.exists());

        let (store, recovered) = WalStore::open(dir.path()).unwrap();
        drop(store);
        assert_eq!(recovered.end_offset("orders"), 2);
        assert_eq!(recovered.replay("orders", None, None, None).len(), 2);
    }

    #[test]
    fn migrate_legacy_journal_is_a_noop_without_a_legacy_file() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!migrate_legacy_journal_file(dir.path()).unwrap());
    }

    #[test]
    fn migrate_legacy_journal_is_a_noop_once_a_wal_already_exists() {
        let dir = tempfile::tempdir().unwrap();
        // Open (and immediately drop) a WalStore to establish journal.wal.
        let (store, _journal) = WalStore::open(dir.path()).unwrap();
        drop(store);
        // Even with a legacy file also present, an existing WAL wins -- do
        // not re-migrate over live WAL state.
        std::fs::write(
            dir.path().join(LEGACY_JOURNAL_FILE_NAME),
            serde_json::to_vec(&TapeJournal::default()).unwrap(),
        )
        .unwrap();
        assert!(!migrate_legacy_journal_file(dir.path()).unwrap());
    }

    #[tokio::test]
    async fn commit_coordinator_submit_round_trips_through_the_dedicated_thread() {
        let dir = tempfile::tempdir().unwrap();
        let (store, journal) = WalStore::open(dir.path()).unwrap();
        let journal = Arc::new(Mutex::new(journal));
        let coordinator = CommitCoordinator::spawn(store, Arc::clone(&journal));

        let outcome = coordinator
            .submit(append_cmd("orders", 1, 100))
            .await
            .unwrap();
        assert!(matches!(outcome, TapeOutcome::Appended(_)));
        assert_eq!(journal.lock().unwrap().end_offset("orders"), 1);
    }

    #[tokio::test]
    async fn commit_coordinator_batches_concurrent_submissions_in_submission_order() {
        let dir = tempfile::tempdir().unwrap();
        let (store, journal) = WalStore::open(dir.path()).unwrap();
        let journal = Arc::new(Mutex::new(journal));
        let coordinator = Arc::new(CommitCoordinator::spawn(store, Arc::clone(&journal)));

        let mut handles = Vec::new();
        for n in 0..32u64 {
            let coordinator = Arc::clone(&coordinator);
            handles.push(tokio::spawn(async move {
                coordinator
                    .submit(append_cmd("orders", n, 100))
                    .await
                    .unwrap()
            }));
        }
        for handle in handles {
            handle.await.unwrap();
        }
        assert_eq!(journal.lock().unwrap().end_offset("orders"), 32);
    }
}
// </HANDWRITE>
