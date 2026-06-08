---
change: refactor-pg-migration
group: refactor-pg-migration
date: 2026-04-07
---

# Requirements

Merge cclab-pg/src/migrate/ useful features (source tracking, ModelDiffer, status reporting) into MigrationRunner in migration.rs. Move Alembic bootstrap to Conductor project layer. Remove migrate/ submodule. Drop Mamba prefix from types (MambaStatus→MigrationStatus, MambaMigrationRunner→remove, MambaMigrationEntry→MigrationEntry, MambaSource→MigrationSource).
