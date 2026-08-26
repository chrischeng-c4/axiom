# ADR 0002 — A backup snapshot seeds a replica only into an empty data directory (#1585, #2468)

Status: Accepted (2026-08-26; recorded from the retired `tech-design/logic/seed-empty-pvcs-from-a-backup-snapshot-before-raft-catch-up.md`).

## Context

A replica whose PersistentVolume was lost, or a cluster being rebuilt from a
whole-journal backup, needs its journal back before it can rejoin the Raft
group. Two designs were available: a live restore RPC that overwrites a
running node's state, or a cold-start step that installs the snapshot before
the Raft host opens its store and then lets ordinary log/snapshot catch-up
finish the job.

The snapshot format already existed: `GET /admin/backup` serves a
`JournalSnapshot`, and `tape backup` ships it to a `file://` or `s3://`
destination through `libs/service-backup`.

## Decision

- Seeding is a cold-start-only operation. `prepare_bootstrap_seed`
  (`src/raft.rs:188`) refuses unconditionally when the data directory
  already carries state, decodes the same `JournalSnapshot` shape that
  `/admin/backup` serves, and atomically writes the per-node applied marker
  and snapshot file that `TapeRaft::from_topology` restores. It is not a
  live restore API.
- `TAPE_BOOTSTRAP_SEED_URI` / `--bootstrap-seed-uri` is accepted only in
  replica mode with a data directory (`src/bin/tape.rs:1034,1079-1097`), and
  the object is fetched through `service_backup::fetch_backup_object`, so
  the seed source is exactly the set of backup destinations.
- Because the operator injects `bootstrapSeedUri` into every pod of a
  `Tape` CR, `serve` consults `data_dir_has_existing_state`
  (`src/raft.rs:145-160`) first and skips seeding when state exists. That
  makes the field mean "bootstrap if empty" across routine pod replacement,
  while `prepare_bootstrap_seed` keeps refusing a populated directory as the
  last line of defence.
- Live replica synchronisation stays with `libs/raft-runtime`; the seed only
  moves the point where catch-up starts (`up_to`) rather than replaying the
  seed as new appends.

## Consequences

- Restoring a cluster is a two-step story: seed each empty node from the
  same snapshot, then let Raft converge. Nothing ever overwrites a running
  replica's durable state through this path.
- A mis-set seed URI on a dirty directory is a loud error, not a silent
  no-op.
- The `/admin/backup` snapshot is a disaster-recovery artefact. It is not
  the Pub/Sub-style subscription snapshot that `ROADMAP.md#seek-snapshot-and-retention`
  will introduce; that outcome owes an ADR resolving the name collision.

## Status of work

Landed. Gate: `cargo test -p tape --test bootstrap --test seed_ha_bootstrap`.
