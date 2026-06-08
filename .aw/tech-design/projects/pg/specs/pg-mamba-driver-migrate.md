---
id: pg-mamba-driver-migrate
fill_sections: [changes]
---

## Changes
<!-- type: changes lang: yaml -->

```yaml
# Full rewrite of `projects/pgkit/mamba-binding/src/{lib,methods,types}.rs`
# on top of `cclab_pg::driver::blocking::{Connection, Transaction,
# MigrationRunner}`. Drops 12 SA-shaped ORM stub symbols; introduces
# 4 driver verbs + Transaction surface (6 methods + context-manager
# form) + a `mambalibs.pg.migrate` sub-module (7 Runner methods + Migration
# constructor). Module name is `mambalibs.pg`; the old `cclab.pg`
# namespace is legacy and must not be used for new Mamba imports.

changes:
  # --- 1. Types: opaque handles for driver/txn/migrate ----------------
  - path: projects/pgkit/mamba-binding/src/types.rs
    action: modify
    description: |
      Replace the current SA-shaped types (`MbPgPool`, `MbQueryBuilder`,
      `MbOrmTable`, `MbColumnType`, `MbColumnDef`, `MbRelationDef`,
      `MbForeignKey`, `MbIndexDef`) with three opaque handle types that
      wrap the blocking-façade types from `cclab_pg::driver::blocking`:

          use std::sync::{Arc, Mutex};
          use cclab_pg::driver::blocking::{
              Connection as PgConnection,
              Transaction as PgTransaction,
              MigrationRunner as PgMigrationRunner,
          };
          use cclab_pg::driver::transaction::IsolationLevel;
          use cclab_pg::migrate::Migration;

          /// Mamba handle for a pooled Postgres connection.
          #[derive(Clone)]
          pub struct MbPgConnection {
              pub(crate) inner: Arc<PgConnection>,
          }

          /// Mamba handle for an in-flight transaction. `Option<>`
          /// because `commit`/`rollback` consume the inner value by
          /// move; we keep the outer handle alive so the context-
          /// manager wrapper can call `.is_completed()` (R2 review
          /// note: commit/rollback are by-value, so the Mamba shim
          /// must take ownership through an `Option::take()`).
          pub struct MbPgTransaction {
              pub(crate) inner: Mutex<Option<PgTransaction>>,
              pub(crate) parent_runtime: Arc<tokio::runtime::Runtime>,
          }

          /// Mamba handle for a migration runner. Wraps one Connection
          /// (Arc-shared with the user-facing `MbPgConnection` when
          /// constructed via `MigrationRunner.from_connection`).
          #[derive(Clone)]
          pub struct MbPgMigrationRunner {
              pub(crate) inner: Arc<PgMigrationRunner>,
          }

          /// Mamba handle for a Migration value object (description +
          /// up_sql + down_sql). Pure data, no IO.
          #[derive(Clone)]
          pub struct MbPgMigration {
              pub(crate) inner: Arc<Migration>,
          }

      No `tokio::runtime::Runtime` is constructed in this file — every
      handle reuses the runtime owned by the underlying
      `cclab_pg::driver::blocking::Connection`. The `parent_runtime`
      field on `MbPgTransaction` is the same `Arc<Runtime>` cloned
      from the parent connection (needed for `Drop` semantics — see
      methods.rs note on `tx.execute`).

  # --- 2. Methods: driver verbs + Transaction + migrate ---------------
  - path: projects/pgkit/mamba-binding/src/methods.rs
    action: modify
    description: |
      Replace every existing `mb_pg_*` function. Final surface (17 funcs
      total — 4 driver + 7 txn + 5 migrate + 1 Migration ctor):

        # Driver
        mb_pg_connect(url: str) -> MbPgConnection
        mb_pg_execute(conn, sql: str) -> int
            # delegates to PgConnection::pool().execute(sql) inside the
            # owned runtime via `conn.runtime().block_on(...)`. Returns
            # rows-affected (sqlx::PgQueryResult::rows_affected as i64).
        mb_pg_ping(conn) -> None
        mb_pg_close(conn) -> None

        # Transaction (R2: mediates `tx.execute` via &mut self on
        # MbPgTransaction. The blocking façade's Transaction owns
        # sqlx::Transaction<'static, Postgres> and is !Send; the Mamba
        # handle uses Mutex<Option<PgTransaction>> for interior mutation
        # — `tx.execute` locks, takes `as_mut_transaction()`, and runs
        # the query through that. We do NOT re-expose the parent pool
        # because doing so would break the txn isolation contract for
        # the very statements the user thinks are inside it.)
        mb_pg_transaction_begin(conn, isolation: str) -> MbPgTransaction
            # isolation ∈ {"read_committed","repeatable_read",
            #               "serializable","read_uncommitted"}
            #   default "read_committed"; unknown -> error.
        mb_pg_transaction_execute(tx, sql: str) -> int
        mb_pg_transaction_savepoint(tx, name: str) -> None
        mb_pg_transaction_rollback_to(tx, name: str) -> None
        mb_pg_transaction_release_savepoint(tx, name: str) -> None
        mb_pg_transaction_commit(tx) -> None
            # Option::take() the inner PgTransaction and consume it.
            # Subsequent ops on the same handle error with
            #   "transaction already completed".
        mb_pg_transaction_rollback(tx) -> None
            # Same Option::take pattern.

        # migrate.MigrationRunner — 7 mounted methods.
        # Why these 7 (Issue Review #1 [Reference Context] note):
        # the blocking façade exposes 14 + load_from_directory. We
        # mount the 7 that are sufficient to drive a full apply/revert
        # workflow from Mamba (init, apply, revert, up, down,
        # applied_migrations, status) plus the value-object ctor
        # (Migration). The other 7 (applied_native_ids, all_entries,
        # applied_migrations_with_details, pending_migrations, migrate,
        # rollback, load_from_directory) are deferred:
        #   - applied_native_ids / all_entries /
        #     applied_migrations_with_details / pending_migrations:
        #     introspection variants subsumed by `status()` for the
        #     basic Mamba workflow.
        #   - migrate / rollback: batch verbs that need a Vec<Migration>
        #     argument — Mamba's list-of-handles ABI lands in a
        #     follow-up issue.
        #   - load_from_directory: filesystem walker that should land
        #     once mamba has a stable Path/Iterator story.
        mb_pg_migration_runner_new(conn, migrations_table: str?) -> MbPgMigrationRunner
        mb_pg_migration_runner_init(runner) -> None
        mb_pg_migration_runner_apply(runner, migration) -> None
        mb_pg_migration_runner_revert(runner, migration) -> None
        mb_pg_migration_runner_up(runner, dir: str) -> list[str]
            # dir interpreted via std::path::PathBuf::from(dir).
        mb_pg_migration_runner_down(runner, dir: str) -> str?
        mb_pg_migration_runner_applied_migrations(runner) -> list[str]
        mb_pg_migration_runner_status(runner, migrations: list) -> dict
            # serialises MigrationStatus { applied, pending,
            #   missing_from_disk } to three list-of-version-strings.

        # Migration value-object ctor
        mb_pg_migration_new(version, description, up_sql, down_sql) -> MbPgMigration

      Every function follows the same error-conversion shape used in
      existing `mb_pg_execute`: wrap `cclab_pg::Result<T>` ->
      `Result<MambaValue, MambaError>` with the `From<DataBridgeError>`
      impl already on `MambaError`. No new `tokio::runtime::Runtime`
      construction in this file (R5 — runtime is owned by Connection).

  # --- 3. Library root: register the new symbol set --------------------
  - path: projects/pgkit/mamba-binding/src/lib.rs
    action: modify
    anchor: 'r.add_symbols(['
    description: |
      Replace the 19-symbol `add_symbols` block (the 17 SA-shaped names
      + the duplicated `table_name_set` helper) with the new surface:

          use crate::methods::{
              // driver
              mb_pg_connect, mb_pg_execute, mb_pg_ping, mb_pg_close,
              // transaction
              mb_pg_transaction_begin, mb_pg_transaction_execute,
              mb_pg_transaction_savepoint, mb_pg_transaction_rollback_to,
              mb_pg_transaction_release_savepoint,
              mb_pg_transaction_commit, mb_pg_transaction_rollback,
              // migrate
              mb_pg_migration_runner_new, mb_pg_migration_runner_init,
              mb_pg_migration_runner_apply, mb_pg_migration_runner_revert,
              mb_pg_migration_runner_up, mb_pg_migration_runner_down,
              mb_pg_migration_runner_applied_migrations,
              mb_pg_migration_runner_status,
              mb_pg_migration_new,
          };

          r.add_symbols([
              // ── driver ─────────────────────────────────────────────
              rt_sym!("connect", mb_pg_connect,
                       "connect(url: str) -> Connection"),
              rt_sym!("execute", mb_pg_execute,
                       "execute(conn, sql: str) -> int"),
              rt_sym!("ping",   mb_pg_ping,
                       "ping(conn) -> None"),
              rt_sym!("close",  mb_pg_close,
                       "close(conn) -> None"),
              // ── Transaction ────────────────────────────────────────
              rt_sym!("transaction_begin",  mb_pg_transaction_begin,
                       "transaction_begin(conn, isolation='read_committed') -> Transaction"),
              rt_sym!("transaction_execute", mb_pg_transaction_execute,
                       "transaction_execute(tx, sql: str) -> int"),
              rt_sym!("transaction_savepoint", mb_pg_transaction_savepoint,
                       "transaction_savepoint(tx, name: str) -> None"),
              rt_sym!("transaction_rollback_to", mb_pg_transaction_rollback_to,
                       "transaction_rollback_to(tx, name: str) -> None"),
              rt_sym!("transaction_release_savepoint",
                       mb_pg_transaction_release_savepoint,
                       "transaction_release_savepoint(tx, name: str) -> None"),
              rt_sym!("transaction_commit",   mb_pg_transaction_commit,
                       "transaction_commit(tx) -> None"),
              rt_sym!("transaction_rollback", mb_pg_transaction_rollback,
                       "transaction_rollback(tx) -> None"),
          ]);

      The 9 migrate symbols are registered under a separate submodule
      `mambalibs.pg.migrate` — see the new file `lib_migrate.rs` below.

  - path: projects/pgkit/mamba-binding/src/lib_migrate.rs
    action: create
    description: |
      New file implementing `PgMigrateMambaModule` that registers under
      the module name `mambalibs.pg.migrate`. Same shape as
      `PgMambaModule` in lib.rs:

          use cclab_mamba_registry::{MambaModule, ModuleRegistrar,
              MAMBA_MODULES, rt_sym};
          use linkme::distributed_slice;

          pub struct PgMigrateMambaModule;

          impl MambaModule for PgMigrateMambaModule {
              fn name(&self) -> &'static str { "mambalibs.pg.migrate" }
              fn doc(&self) -> &'static str {
                  "Mamba bindings for cclab-pg::migrate — schema versioning"
              }
              fn register(&self, r: &mut ModuleRegistrar) {
                  use crate::methods::{
                      mb_pg_migration_runner_new,
                      mb_pg_migration_runner_init,
                      mb_pg_migration_runner_apply,
                      mb_pg_migration_runner_revert,
                      mb_pg_migration_runner_up,
                      mb_pg_migration_runner_down,
                      mb_pg_migration_runner_applied_migrations,
                      mb_pg_migration_runner_status,
                      mb_pg_migration_new,
                  };
                  r.add_symbols([
                      rt_sym!("MigrationRunner",  mb_pg_migration_runner_new,
                               "MigrationRunner(conn, migrations_table=None) -> Runner"),
                      rt_sym!("runner_init",      mb_pg_migration_runner_init,
                               "runner_init(runner) -> None"),
                      rt_sym!("runner_apply",     mb_pg_migration_runner_apply,
                               "runner_apply(runner, migration) -> None"),
                      rt_sym!("runner_revert",    mb_pg_migration_runner_revert,
                               "runner_revert(runner, migration) -> None"),
                      rt_sym!("runner_up",        mb_pg_migration_runner_up,
                               "runner_up(runner, dir: str) -> list[str]"),
                      rt_sym!("runner_down",      mb_pg_migration_runner_down,
                               "runner_down(runner, dir: str) -> str?"),
                      rt_sym!("runner_applied_migrations",
                               mb_pg_migration_runner_applied_migrations,
                               "runner_applied_migrations(runner) -> list[str]"),
                      rt_sym!("runner_status",    mb_pg_migration_runner_status,
                               "runner_status(runner, migrations: list) -> dict"),
                      rt_sym!("Migration",        mb_pg_migration_new,
                               "Migration(version, description, up_sql, down_sql) -> Migration"),
                  ]);
              }
          }

          #[distributed_slice(MAMBA_MODULES)]
          static PG_MIGRATE_MAMBA_MODULE: &dyn MambaModule =
              &PgMigrateMambaModule;

      Add `pub mod lib_migrate;` to lib.rs so the registrar slot is
      linked.

  # --- 4. Cargo manifest: depend on cclab_pg::driver only --------------
  - path: projects/pgkit/mamba-binding/Cargo.toml
    action: modify
    anchor: 'cclab-pg ='
    description: |
      Keep the dependency line as-is (`cclab-pg = { path = "../pg" }`).
      The boundary is enforced at the source level — no Cargo feature
      gate. Confirm that no `cclab-pg-orm` style feature is enabled.
      Add `sqlx = { workspace = true }` only if the pool-execute path
      cannot import `Executor` via the re-export at
      `cclab_pg::driver::*` — verify in implementation.

  # --- 5. Mamba-side boundary audit test -------------------------------
  - path: projects/pgkit/mamba-binding/tests/test_layer_boundaries.rs
    action: create
    description: |
      Sibling of `projects/pgkit/pg/tests/test_layer_boundaries.rs`
      (Issue Review #1 [Scope] note pinned this to a separate file —
      the existing test is `CARGO_MANIFEST_DIR`-scoped to pg/src/driver
      and cannot grep pg-mamba's sources). Walks
      `$CARGO_MANIFEST_DIR/src/` recursively and fails if any `.rs`
      file contains the substrings:

          "cclab_pg::orm::"
          "use cclab_pg::orm::"

      The test mirrors the shape of the pg-side boundary test
      (FORBIDDEN list, walk closure, panic-on-read-error). Inline
      module doc states the Phase B story: once orm/migrate split into
      separate crates, this grep retires and the Cargo manifest's
      missing `cclab-pg-orm` dep becomes the enforcement.

  # --- 6. Integration tests (real Postgres, #[ignore]) -----------------
  - path: projects/pgkit/mamba-binding/tests/test_blocking_binding.rs
    action: create
    description: |
      Three `#[ignore]` integration tests against a real Postgres
      (mirrors `projects/pgkit/pg/tests/test_blocking.rs` shape; run
      with `cargo test -p cclab-pg-mamba -- --ignored` against a local
      Postgres at `$DATABASE_URL`):

      1. `pool_is_reused_across_executes` (R1 verification)
         - Call `mb_pg_connect(url)` once.
         - Loop 100×: `mb_pg_execute(conn, "SELECT 1")`.
         - Assert: no panic, each returns rows_affected==0 (SELECT has
           no row count), and `Arc::strong_count(&conn.inner)` stays
           constant at 1 across the loop (proves no per-call clone).

      2. `transaction_savepoint_rollback_to` (R2 verification)
         - CREATE TEMPORARY TABLE marker(id int).
         - `tx = transaction_begin(conn, "read_committed")`.
         - INSERT 1; `tx_savepoint(tx, "sp")`; INSERT 2;
           `tx_rollback_to(tx, "sp")`; `tx_commit(tx)`.
         - SELECT count(*) FROM marker; assert == 1.

      3. `migrate_apply_then_revert` (R3 verification)
         - `runner = MigrationRunner(conn, None)`; `runner_init(runner)`.
         - `m = Migration("0001", "test", "CREATE TABLE t(id int)",
              "DROP TABLE t")`.
         - `runner_apply(runner, m)`;
           assert "0001" in `runner_applied_migrations(runner)`.
         - `runner_revert(runner, m)`;
           assert "0001" NOT in `runner_applied_migrations(runner)`.

      Tests use the Mamba runtime via the
      `cclab-mamba-registry`-provided test harness shape already used
      by other mamba binding tests (look up the canonical entry point;
      do not invent a new one).

  # --- 7. Acceptance gate ----------------------------------------------
  - path: VERIFY
    action: verify
    description: |
      All must pass in order:

        cargo check  -p cclab-pg-mamba
        cargo build  -p cclab-pg-mamba
        cargo test   -p cclab-pg-mamba --lib
        cargo test   -p cclab-pg-mamba --test test_layer_boundaries
        cargo check  --workspace

      Mamba binding source must not reference the orm layer:

        ! grep -rn 'cclab_pg::orm::' \
            projects/pgkit/mamba-binding/src

      The 12 dropped SA-shaped symbols must disappear from the surface
      (sanity grep — none of these should resolve in lib.rs's
      `add_symbols([…])` block):

        ! grep -E 'QueryBuilder|DeclarativeBase|mapped_column|relationship|ForeignKey|Index|String|Text|JSON|UUID|DateTime' \
            projects/pgkit/mamba-binding/src/lib.rs

      Optional (only when a Postgres is available at $DATABASE_URL):

        cargo test  -p cclab-pg-mamba -- --ignored

      All three #[ignore] integration tests must pass against the real
      database.
```

# Reviews

### Review 1
**Verdict:** approved

- [changes] Each unit has a concrete `path`, `action` (`modify` / `create` / `verify`), and a description precise enough to execute mechanically. The unit ordering is dependency-correct: types.rs (the data model) → methods.rs (the verbs over those types) → lib.rs / lib_migrate.rs (registrar surface) → Cargo.toml (deps) → boundary test → real-DB integration tests → acceptance gate. At no intermediate step does the workspace fail to build because the spec calls for a single PR that flips all of `lib.rs`'s `add_symbols` + `methods.rs` + `types.rs` together.
- [changes] The three Issue Review #1 concerns are addressed in-spec, in line with the reviewer's "do not leave open" requirement: (a) R2 `tx.execute` ambiguity is resolved to the `Mutex<Option<PgTransaction>>` + `as_mut_transaction()` form, with a one-paragraph justification of why re-exposing the parent pool would break isolation (unit 2); (b) the boundary test is committed to a sibling file under `projects/pgkit/mamba-binding/tests/` rather than extending the pg-side test (unit 5); (c) the 7 mounted `MigrationRunner` methods are enumerated with a per-deferred-method rationale (unit 2 — covers `applied_native_ids`, `all_entries`, `applied_migrations_with_details`, `pending_migrations`, `migrate`, `rollback`, `load_from_directory`).
- [changes] The acceptance gate in unit 7 is mechanically checkable: `cargo check -p cclab-pg-mamba`, `cargo build -p cclab-pg-mamba`, `cargo test -p cclab-pg-mamba --lib`, `cargo test -p cclab-pg-mamba --test test_layer_boundaries`, `cargo check --workspace`, plus two negative greps (no `cclab_pg::orm::` reference in `mamba-binding/src/`; none of the 12 dropped SA-shape symbol names resolve in `lib.rs`'s `add_symbols([…])`), plus an optional `--ignored` integration suite that exercises R1/R2/R3 against a real Postgres. Verification is binary; no human-judgement gate is left in the loop.
