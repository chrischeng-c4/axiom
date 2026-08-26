# ADR 0003 — The single-node WAL logs `TapeCommand` frames behind one group-commit fsync (#3052)

Status: Accepted (2026-08-26; recorded from the retired `tech-design/src/tape/work_items/replace_the_single_node_whole_file_journal_rewrite_with_an_appen.py`).

## Context

`tape serve --data-dir` used to persist by rewriting and fsyncing the whole
JSON journal on every mutating request. Load testing measured a flat
85–89 ops/s ceiling independent of concurrency: the per-request fsync on a
whole-file rewrite was the bottleneck, not lock contention.

Two representations were on the table for an append-only replacement: log
the journal's post-mutation state, or log the command that caused it.
`TapeJournal::append_at` enforces retention as a side effect of appending
and can delete events, so post-state frames would silently drop the history
a replay needs to reconstruct checkpoints deterministically.

## Decision

- A WAL frame encodes exactly one `TapeCommand`, never `TapeJournal`
  state, and recovery replays frames through the same `apply_command`
  free function that the Raft state machine applies through
  (`src/wal.rs` module doc, `src/raft.rs`). The single-node and
  replicated paths therefore cannot drift in how a command mutates the
  journal.
- Group commit: `WalStore::commit` appends every command in a batch, issues
  **one** fsync covering the batch, and only then takes the journal lock to
  apply the batch in order. The lock is never held across the fsync, and
  `FsyncPolicy::Always` is never downgraded — the design changes how many
  commands one fsync covers, never whether a covering fsync precedes the
  ack.
- A batch fails closed. On any append or sync failure no command in it is
  applied, the caller receives the `ErrorKind` and `errno` intact so ENOSPC
  and EIO can latch degraded read-only mode, and the store poisons itself
  until reopened so a partially landed batch can never be replayed twice.
- Layout is fixed: `<dir>/journal.wal` plus `<dir>/journal-<seq>.snap`
  through `libs/storage-durable`'s `FramedLogWriter`, `FramedLogReader` and
  `SnapshotFileStore`. Torn-tail truncation is a property of those
  primitives, not logic reimplemented here.
- Snapshot and backup wire formats, the Raft-replicated path, and the
  legacy `--store <file>` offline CLI verbs are unchanged.

## Consequences

- Every acknowledged mutation survives a process crash with RPO = 0, and
  `durable_crash_recovery` measures that under SIGKILL rather than trusting
  the design.
- `tape_perf_gate` is the local throughput ceiling for this path and is the
  only performance claim tape makes about itself (see
  `ROADMAP.md#peer-broker-benchmarks`).
- New `TapeCommand` variants must be replayable through `apply_command`;
  a variant that only the HTTP layer understands would break recovery.

## Status of work

Landed. Gate: `cargo test -p tape --test durable_write_path --test durable_crash_recovery`
and `cargo test --release -p tape --test tape_perf_gate`.
