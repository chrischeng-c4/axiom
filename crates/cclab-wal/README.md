# Cclab Wal

## Brief

Cclab Wal is the shared Rust write-ahead-log implementation for cclab storage
engines.

It owns the reusable WAL entry/header format, CRC-backed corruption detection,
buffered append and fsync behavior, file rotation, typed replay, WAL discovery,
and old-segment cleanup helpers. The public surface is a Rust library API used
by higher-level storage crates.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Entry Format And Corruption Detection | - | implemented | passing | conformance | not_ready | WAL header/version, JSON entry encoding, length prefix, checksum, and corruption errors |
| Durable Writer And Rotation | - | implemented | passing | conformance | not_ready | append, buffered flush/fsync, position tracking, flush interval, and file rotation |
| Replay Reader And File Retention | - | implemented | passing | conformance | not_ready | replay reader, iterator, WAL file discovery, and cleanup helpers |

### Entry Format And Corruption Detection

ID: entry-format-and-corruption-detection
Type: RuntimeTool
Surfaces: Rust API: `cclab_wal::{WalEntry, WalHeader, WalError, Result}`
EC Dimensions: behavior: `cargo test -p cclab-wal` - header roundtrip, entry roundtrip, checksum validation, invalid magic/version/corruption errors
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Wal defines a reusable WAL file and entry format with versioned headers, JSON-encoded typed operations, length prefixes, CRC32 checksums, and explicit corruption errors.
Gate Inventory: `cargo test -p cclab-wal`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Entry format and checksum contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-wal` |

### Durable Writer And Rotation

ID: durable-writer-and-rotation
Type: RuntimeTool
Surfaces: Rust API: `cclab_wal::{WalWriter, WalConfig}`
EC Dimensions: behavior: `cargo test -p cclab-wal` - writer creation, append position, flush/fsync state, rotation, and file creation
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Wal writes typed storage operations to durable WAL files with buffered appends, explicit flush/fsync control, position tracking, flush interval checks, and size-based rotation.
Gate Inventory: `cargo test -p cclab-wal`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Writer durability and rotation contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-wal` |

### Replay Reader And File Retention

ID: replay-reader-and-file-retention
Type: RuntimeTool
Surfaces: Rust API: `cclab_wal::{WalReader, find_wal_files, cleanup_old_wal_files}`
EC Dimensions: behavior: `cargo test -p cclab-wal` - empty replay, multi-entry replay, iterator replay, WAL file discovery, and retention cleanup helpers
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Wal replays typed operations from WAL files through pull and iterator APIs, discovers WAL files in timestamp order, and exposes cleanup helpers for old WAL segments.
Gate Inventory: `cargo test -p cclab-wal`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Reader replay and retention contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-wal` |
