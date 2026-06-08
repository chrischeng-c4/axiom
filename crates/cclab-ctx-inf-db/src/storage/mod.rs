//! Storage layer — WAL, snapshots, recovery, and the durability contract.
//!
//! # Durability model (write-behind)
//!
//! `cclab-ctx-inf-db` uses a **write-behind** persistence architecture. All
//! mutations issued through [`crate::engine::CtxInfEngine`] (`create_entity`,
//! `update_relation`, etc.) return to the caller **before** the operation has
//! reached stable storage. The mutation is applied to in-memory DashMaps
//! synchronously and then enqueued onto a bounded crossbeam channel consumed
//! by a single background OS thread ([`handle::PersistenceHandle`]). The
//! background thread is the only writer to the WAL — no other thread appends
//! or fsyncs. See `handle.rs::PersistenceHandle::log_operation` (~L114–120)
//! and `handle.rs::persistence_thread` (~L142–227).
//!
//! # fsync policy and RPO window
//!
//! The background thread fsyncs the WAL when **any** of the following occurs:
//! (1) an explicit [`crate::engine::CtxInfEngine::flush`] request is drained
//! from the channel (`handle.rs` ~L188–195); (2) the underlying
//! [`cclab_wal::WalWriter`] reports `should_flush()` after an append
//! (`handle.rs` ~L179–185); or (3) the recv-timeout elapses and
//! `wal_flush_interval_ms` has passed since `last_flush` (`handle.rs`
//! ~L202–212). Between fsyncs, committed-to-RAM mutations have not reached
//! disk. The **recovery point objective (RPO)** on a crash is therefore up to
//! `PersistenceConfig::wal_flush_interval_ms` (default 100 ms;
//! [`handle::PersistenceConfig::default`] at `handle.rs` ~L40–49).
//!
//! # Corruption handling
//!
//! The WAL is CRC32-framed per entry (format owned by `cclab-wal`). On
//! recovery, a corrupted entry — bad CRC, truncated mid-entry, or an
//! unrecoverable read error — causes replay of **that file** to stop at the
//! corrupt position; earlier entries in the file are kept, later entries are
//! discarded. See `recovery.rs::RecoveryManager::replay_wal` (~L147–185). The
//! count is reported via [`recovery::RecoveryStats::corrupted_entries`].
//! Snapshots are protected by a SHA-256 checksum over the JSON payload; a
//! mismatch during load returns an error and aborts recovery (see
//! `snapshot.rs::SnapshotLoader::load_latest` ~L252–261).
//!
//! # Recovery guarantee
//!
//! On startup, [`recovery::RecoveryManager::recover`] restores state in two
//! steps: (1) load the latest valid snapshot (if any) and rebuild entities,
//! relations, adjacency, and type index; (2) replay every WAL entry whose
//! captured position is at or after the snapshot's `wal_position`. Recovery
//! is **prefix-consistent** — the restored engine reflects exactly the
//! durable prefix of the operation log up to the first corrupt entry in each
//! WAL file. See `recovery.rs::RecoveryManager::recover` (~L39–143).
//!
//! # Snapshot atomicity
//!
//! Snapshots are written via a `.tmp` → `.snap` filesystem rename after a
//! payload fsync, so readers never observe a partially written snapshot (see
//! `snapshot.rs::SnapshotWriter::create` ~L145–223). The captured WAL
//! position is embedded in the snapshot header so recovery can skip the
//! already-snapshotted WAL prefix.

pub mod handle;
pub mod page;
pub mod recovery;
pub mod snapshot;
pub mod wal_ops;

pub use handle::{PersistenceConfig, PersistenceHandle};
pub use recovery::{RecoveryManager, RecoveryStats};
pub use wal_ops::GraphOp;
