---
number: 1196
title: "Refactor: remove Conductor-specific migration code from cclab-pg"
state: open
labels: [crate:pg, priority:p2, type:refactor]
group: "refactor-pg-migration"
---

# #1196 — Refactor: remove Conductor-specific migration code from cclab-pg

## Problem

`cclab-pg` is a Layer 2 library but contains Conductor-specific migration code in `src/migrate/`:

- `MambaMigrationRunner` — naming tied to Conductor/Mamba
- Alembic bootstrap logic (`bootstrap_alembic`) — one-time Conductor migration from Python/Alembic
- `_mamba_migrations` table with `source IN ('alembic', 'mamba')` — Conductor-specific schema
- `MambaSource`, `MambaStatus`, `MambaMigrationEntry` — all Mamba-prefixed types

## Expected

Layer 2 libraries should be project-agnostic. The useful generic capabilities should be absorbed into the existing `MigrationRunner`, and Conductor-specific concerns should live in the Conductor project layer.

## Plan

1. **Merge useful features into `MigrationRunner`** — `source` tracking, `ModelDiffer`, status reporting
2. **Move Alembic bootstrap out** — into Conductor project layer or a one-time migration SQL file
3. **Remove `migrate/` submodule** — after merging into the generic engine
4. **Rename types** — drop `Mamba` prefix (e.g. `MambaStatus` → reuse `MigrationStatus`)

## Files

- `crates/cclab-pg/src/migrate/mod.rs`
- `crates/cclab-pg/src/migrate/runner.rs`
- `crates/cclab-pg/src/migrate/model_diff.rs`
- `crates/cclab-pg/src/migrate/status.rs`
- `crates/cclab-pg/src/migration.rs` (absorb into)
