---
id: pg-blocking-facade-row-bulk-executor
fill_sections: [changes]
---

## Changes
<!-- type: changes lang: yaml -->

```yaml
# Closes the NON_PARITY_FILES carve-out for `driver/row.rs`,
# `driver/bulk.rs`, and `driver/executor.rs` by adding blocking
# facade siblings under `driver/blocking/`. The async surface is
# treated as fixed (issue R2/R5 — additive only; no async-side
# signature edits). Three new files + one mod.rs update + two test
# edits land in a single change.
#
# TD-time decisions resolved here:
#
#   Row shape: Row is a pure data carrier (HashMap<String,
#     ExtractedValue>); the 22 async methods on the async-side
#     `impl Row` are mostly free-static-style ctors / queries that
#     take `&PgPool` (or `executor: E: sqlx::Executor`) plus
#     trailing args. The blocking siblings are added as additional
#     `impl Row` methods in `driver/blocking/row.rs` and take
#     `&blocking::Connection` instead of the executor/pool. Body is
#     a single `conn.runtime().block_on(crate::driver::Row::<fn>(
#     conn.as_async().pool(), ...))` expression. No new struct is
#     introduced — the parity walker matches on short type name
#     `Row` and an additional `impl Row` block in the blocking
#     subtree contributes its methods to the blocking surface.
#
#   BulkExecutor shape: `blocking::BulkExecutor` is a new owning
#     struct holding `{ inner: crate::driver::BulkExecutor, rt:
#     Arc<Runtime> }`. The async-side BulkExecutor already takes a
#     `&Connection + BulkConfig` ctor and three `&self` async
#     methods (`insert_parallel`, `update_parallel`,
#     `delete_parallel`). The blocking sibling mirrors that shape
#     1:1 via `rt.block_on(self.inner.<fn>(...))`. BulkConfig is a
#     sync builder; no wrapper is needed (the parity walker only
#     compares `pub async fn` on the async side, so BulkConfig is
#     out of scope).
#
#   QueryExecutor shape: `blocking::QueryExecutor<'a>` holds `{
#     inner: crate::driver::QueryExecutor<'a>, rt: Arc<Runtime> }`.
#     The async ctors `new(pool: &'a PgPool)` and
#     `with_config(pool: &'a PgPool, cfg: ExecutorConfig)` are
#     mirrored as `new(conn: &'a blocking::Connection)` /
#     `with_config(conn: &'a blocking::Connection, cfg:
#     ExecutorConfig)` — both pull the pool via
#     `conn.as_async().pool()` and clone the runtime. The generic
#     `fetch_all<T, F>` signature is preserved verbatim on the
#     blocking side; `F: Fn(...) -> ... + Clone` is kept because
#     the closure crosses into `block_on` once per call and the
#     `Clone` bound is harmless when called once. `execute` is a
#     straight pass-through.
#
#   Executor-generic collapse (R4): the async-side `Row::insert<'a,
#     E>` etc. are generic over `E: sqlx::Executor`. The blocking
#     facade collapses the generic away by always passing
#     `conn.as_async().pool()` (which implements Executor) — the
#     facade fn is non-generic and takes only `&blocking::
#     Connection`. This codifies the design principle that the
#     blocking surface is concrete-only and avoids re-exporting
#     `sqlx::Executor` from `cclab_pg::driver::blocking::*`.
#
#   ALLOWLIST additions (test_blocking_facade_shape.rs): three
#     sync-only constructors on the new types — `BulkExecutor::
#     new`, `QueryExecutor::new`, `QueryExecutor::with_config`.
#     Each is a pure wrapper around the async-side sync ctor (no
#     IO, no block_on needed); they match the existing pattern of
#     `Connection::from_parts` and `MigrationRunner::new` on the
#     ALLOWLIST today.
#
#   Acceptance gate scope (R8): `cargo check --workspace` is
#     explicitly NOT part of the gate. The pre-existing score-cli
#     template-path breakage is outside this scope and would
#     spuriously fail an otherwise-clean change. Per-crate
#     `cargo check -p cclab-pg` + the two audit tests + lib + layer
#     boundaries are the binding gate.

changes:
  # --- 1. Row blocking entry points --------------------------------
  - path: projects/pgkit/pg/src/driver/blocking/row.rs
    action: create
    description: |
      New file under `driver/blocking/`. Adds 22 blocking sibling
      methods on `impl Row` (additional inherent impl block — the
      async-side `Row` struct is reused; no new type). The parity
      walker matches by short type name `Row`, so any inherent
      `impl Row` block under `driver/blocking/` contributes its
      methods to the blocking surface.

      Layout:

        // HANDWRITE-BEGIN reason: blocking-facade wrappers for
        //   driver::Row — same codegen gap as the parity tests
        //   themselves. Closes when score grows a `facade-mirror`
        //   section type that emits `rt.block_on(async_fn(...))`
        //   wrappers from an async signature.
        //! @spec
        //!   .aw/tech-design/projects/pg/specs/pg-blocking-facade-row-bulk-executor.md#changes
        use crate::driver::Row;
        use crate::driver::blocking::Connection;
        use crate::driver::{
            ExtractedValue, ColumnUpdate, M2MOptions, CascadeOptions,
            CheckedDeleteOptions, RelationDescriptor, EagerLoadOptions,
        };
        use crate::Result;
        use std::collections::HashMap;

        impl Row {
            pub fn insert_blocking(
                conn: &Connection,
                table: &str,
                values: &[(String, ExtractedValue)],
            ) -> Result<Self> {
                conn.runtime().block_on(
                    Row::insert(conn.as_async().pool(), table, values)
                )
            }

            pub fn insert_many_blocking(
                conn: &Connection,
                table: &str,
                rows: &[Vec<(String, ExtractedValue)>],
            ) -> Result<Vec<Self>> {
                conn.runtime().block_on(
                    Row::insert_many(conn.as_async().pool(), table, rows)
                )
            }

            // ... 20 more methods, one per async sibling. Each
            //     follows the same shape:
            //
            //       pub fn <name>_blocking(conn: &Connection, ...) -> Result<T> {
            //           conn.runtime().block_on(
            //               Row::<name>(conn.as_async().pool(), ...)
            //           )
            //       }
            //
            //     The `_blocking` suffix differentiates from the
            //     async-side methods of the same name; the parity
            //     walker strips the suffix for matching. Wait — see
            //     R6 note below: the walker matches verbatim on
            //     short-name. So instead of `_blocking` suffix, the
            //     blocking entries reuse the EXACT async name. Rust
            //     allows duplicate fn names across separate `impl T`
            //     blocks as long as the SCOPE differs — the
            //     async-side impl is in `crate::driver::row` and the
            //     blocking-side impl is in `crate::driver::blocking::
            //     row`, but both target the same nominal type `Row`
            //     and therefore live in the same method namespace.
            //
            //     Resolution: the blocking entries take a different
            //     LEADING ARGUMENT TYPE (`&Connection` vs.
            //     `&PgPool` / `executor: E`). For Rust's method
            //     resolution, the two cannot collide because Rust
            //     does NOT allow two inherent `fn` items with the
            //     same name on the same type even if their argument
            //     types differ. We MUST use distinct names.
            //
            //     Decision: use the `_blocking` suffix on every
            //     entry. Update the parity walker to strip the
            //     `_blocking` suffix when matching blocking → async.
            //     This is a one-line change in
            //     `test_async_blocking_parity.rs` and is the
            //     minimal cost solution.
        }
        // HANDWRITE-END

      The 22 async methods to mirror (from
      `projects/pgkit/pg/src/driver/row.rs`):

        insert, insert_many, upsert, upsert_many, find_by_id,
        find_many, update, delete, count, find_with_relations,
        find_many_with_relations, find_one_eager,
        delete_with_cascade, delete_checked, create_join_table,
        add_m2m_relation, remove_m2m_relation,
        clear_m2m_relations, fetch_m2m_related,
        count_m2m_related, has_m2m_relation, set_m2m_relations

      Naming convention: every blocking entry uses the
      `<async_name>_blocking` suffix. The parity walker is
      extended to recognise the suffix when pairing.

  # --- 2. BulkExecutor blocking sibling ----------------------------
  - path: projects/pgkit/pg/src/driver/blocking/bulk.rs
    action: create
    description: |
      New owning blocking type that wraps `crate::driver::
      BulkExecutor` with an `Arc<Runtime>`. Three blocking siblings
      for the three async methods on the async-side type.

      Layout:

        // HANDWRITE-BEGIN reason: see row.rs above — same
        //   facade-mirror codegen gap.
        //! @spec
        //!   .aw/tech-design/projects/pg/specs/pg-blocking-facade-row-bulk-executor.md#changes
        use std::sync::Arc;
        use tokio::runtime::Runtime;

        use crate::driver::BulkExecutor as AsyncBulkExecutor;
        use crate::driver::{BulkConfig, BulkResult, ColumnUpdate};
        use crate::driver::blocking::Connection;
        use crate::Result;

        /// Blocking sibling for `crate::driver::BulkExecutor`.
        ///
        /// Wraps an owned async `BulkExecutor` plus a clone of the
        /// `Connection`'s runtime so each method can be a single
        /// `rt.block_on(inner.<fn>(...))` pass-through.
        pub struct BulkExecutor {
            inner: AsyncBulkExecutor,
            rt: Arc<Runtime>,
        }

        impl BulkExecutor {
            pub fn new(conn: &Connection, config: BulkConfig) -> Self {
                Self {
                    inner: AsyncBulkExecutor::new(conn.as_async(), config),
                    rt: conn.runtime(),
                }
            }

            pub fn insert_parallel(
                &self,
                table: &str,
                rows: &[Vec<(String, crate::driver::ExtractedValue)>],
            ) -> Result<BulkResult> {
                self.rt.block_on(self.inner.insert_parallel(table, rows))
            }

            pub fn update_parallel(
                &self,
                table: &str,
                updates: &[(i64, Vec<ColumnUpdate>)],
            ) -> Result<BulkResult> {
                self.rt.block_on(self.inner.update_parallel(table, updates))
            }

            pub fn delete_parallel(
                &self,
                table: &str,
                ids: &[i64],
            ) -> Result<BulkResult> {
                self.rt.block_on(self.inner.delete_parallel(table, ids))
            }
        }
        // HANDWRITE-END

      Note: `BulkExecutor::new` is a sync ctor (no IO until first
      method call). Added to `test_blocking_facade_shape.rs`
      ALLOWLIST.

  # --- 3. QueryExecutor blocking sibling ---------------------------
  - path: projects/pgkit/pg/src/driver/blocking/executor.rs
    action: create
    description: |
      New owning blocking type that wraps `crate::driver::
      QueryExecutor<'a>` with a lifetime + `Arc<Runtime>`.
      Preserves the `fetch_all<T, F>` generic surface per issue R6.

      Layout:

        // HANDWRITE-BEGIN reason: see row.rs above.
        //! @spec
        //!   .aw/tech-design/projects/pg/specs/pg-blocking-facade-row-bulk-executor.md#changes
        use std::sync::Arc;
        use tokio::runtime::Runtime;

        use sqlx::FromRow;
        use sqlx::postgres::{PgArguments, PgRow};
        use sqlx::query::Query;
        use sqlx::Postgres;

        use crate::driver::QueryExecutor as AsyncQueryExecutor;
        use crate::driver::ExecutorConfig;
        use crate::driver::blocking::Connection;
        use crate::Result;

        /// Blocking sibling for `crate::driver::QueryExecutor<'a>`.
        ///
        /// Borrows from the `Connection` for the lifetime `'a` and
        /// shares its runtime.
        pub struct QueryExecutor<'a> {
            inner: AsyncQueryExecutor<'a>,
            rt: Arc<Runtime>,
        }

        impl<'a> QueryExecutor<'a> {
            pub fn new(conn: &'a Connection) -> Self {
                Self {
                    inner: AsyncQueryExecutor::new(conn.as_async().pool()),
                    rt: conn.runtime(),
                }
            }

            pub fn with_config(conn: &'a Connection, config: ExecutorConfig) -> Self {
                Self {
                    inner: AsyncQueryExecutor::with_config(
                        conn.as_async().pool(),
                        config,
                    ),
                    rt: conn.runtime(),
                }
            }

            pub fn fetch_all<T, F>(&self, sql: &str, bind_fn: F) -> Result<Vec<T>>
            where
                T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
                F: Fn(Query<'_, Postgres, PgArguments>)
                    -> Query<'_, Postgres, PgArguments>
                    + Clone,
            {
                self.rt.block_on(self.inner.fetch_all(sql, bind_fn))
            }

            pub fn execute(&self, sql: &str) -> Result<u64> {
                self.rt.block_on(self.inner.execute(sql))
            }
        }
        // HANDWRITE-END

      Implementation note for R6: if `Clone` on `F` causes a
      compile error inside `block_on` (the closure must outlive
      the `block_on` call), document the concession in the doc
      comment and accept the divergence; the facade still
      type-checks because the parity walker only verifies fn
      names, not full signatures.

      Note: `QueryExecutor::new` and `QueryExecutor::with_config`
      are sync ctors. Both added to
      `test_blocking_facade_shape.rs` ALLOWLIST.

  # --- 4. blocking/mod.rs wiring -----------------------------------
  - path: projects/pgkit/pg/src/driver/blocking/mod.rs
    action: modify
    anchor: 'mod transaction;'
    description: |
      Add three module declarations + two re-exports. After the
      change the module section reads:

        mod bulk;
        mod connection;
        mod executor;
        mod migration;
        mod row;
        mod transaction;

        pub use bulk::BulkExecutor;
        pub use connection::Connection;
        pub use executor::QueryExecutor;
        pub use migration::MigrationRunner;
        pub use transaction::Transaction;

      `Row` is not re-exported from `blocking::*` because the
      async-side `Row` is already re-exported from `crate::driver`
      and the new methods attach to that same type. Users call
      `cclab_pg::driver::Row::insert_blocking(&conn, ...)` —
      identical short name, distinct method name.

  # --- 5. Parity test carve-out narrowing --------------------------
  - path: projects/pgkit/pg/tests/test_async_blocking_parity.rs
    action: modify
    anchor: 'const NON_PARITY_FILES: &[&str] = &['
    description: |
      Remove three entries — `bulk.rs`, `executor.rs`, `row.rs` —
      from `NON_PARITY_FILES`. After the change the constant reads:

        const NON_PARITY_FILES: &[&str] = &[
            // migrate/ auxiliary types — not yet in the blocking
            // facade scope. Narrowing this is a future follow-up.
            "history_vis.rs",
            "model_diff.rs",
            "status_report.rs",
        ];

      Update the surrounding doc comment to drop the
      `driver/ ergonomic helpers` paragraph (now resolved). The
      remaining three entries are the deliberate residual carve-out
      for the migrate auxiliary types.

      Additionally, extend the walker to strip the `_blocking`
      suffix when pairing blocking-side methods to async siblings.
      The exact pattern: in `collect_pub_methods` (or wherever the
      blocking-surface walker emits short names), insert:

        let name = match name.strip_suffix("_blocking") {
            Some(stripped) => stripped.to_string(),
            None => name,
        };

      so that `Row::insert_blocking` pairs with the async-side
      `Row::insert`. This is the minimal code change needed to make
      the 22 Row blocking entries satisfy R1 without renaming any
      async-side method.

  # --- 6. Facade-shape test ALLOWLIST extension --------------------
  - path: projects/pgkit/pg/tests/test_blocking_facade_shape.rs
    action: modify
    anchor: 'const ALLOWLIST: &[(&str, &str, &str)] = &['
    description: |
      Append three entries for the new sync-only constructors:

        ("BulkExecutor", "new", "constructor: wraps async BulkExecutor + clones runtime (no IO)"),
        ("QueryExecutor", "new", "constructor: borrows pool from Connection (no IO)"),
        ("QueryExecutor", "with_config", "constructor: same as new + ExecutorConfig (no IO)"),

      No other ALLOWLIST changes are required. The 22 Row blocking
      entries and the 3 BulkExecutor + 2 QueryExecutor IO methods
      all satisfy the existing `block_on`-must-appear rule (each
      body is a single `rt.block_on(inner.<fn>(...))` expression).

  # --- 7. Audit findings -------------------------------------------
  - path: AUDIT
    action: verify
    description: |
      R7 audit of the post-change parity surface for the three
      target files. The parity test must pass against this
      baseline; any drift is a violation.

  audit_findings:
    driver_row:
      async_surface_count: 22
      blocking_surface_count: 22
      blocking_entries:
        - "Row::insert_blocking"
        - "Row::insert_many_blocking"
        - "Row::upsert_blocking"
        - "Row::upsert_many_blocking"
        - "Row::find_by_id_blocking"
        - "Row::find_many_blocking"
        - "Row::update_blocking"
        - "Row::delete_blocking"
        - "Row::count_blocking"
        - "Row::find_with_relations_blocking"
        - "Row::find_many_with_relations_blocking"
        - "Row::find_one_eager_blocking"
        - "Row::delete_with_cascade_blocking"
        - "Row::delete_checked_blocking"
        - "Row::create_join_table_blocking"
        - "Row::add_m2m_relation_blocking"
        - "Row::remove_m2m_relation_blocking"
        - "Row::clear_m2m_relations_blocking"
        - "Row::fetch_m2m_related_blocking"
        - "Row::count_m2m_related_blocking"
        - "Row::has_m2m_relation_blocking"
        - "Row::set_m2m_relations_blocking"
      gaps:
        async_missing_blocking: []
        blocking_missing_async: []
    driver_bulk:
      async_surface_count: 3
      blocking_surface_count: 3
      blocking_entries:
        - "BulkExecutor::insert_parallel"
        - "BulkExecutor::update_parallel"
        - "BulkExecutor::delete_parallel"
      blocking_ctor: "BulkExecutor::new (ALLOWLIST)"
      gaps:
        async_missing_blocking: []
        blocking_missing_async: []
    driver_executor:
      async_surface_count: 2
      blocking_surface_count: 2
      blocking_entries:
        - "QueryExecutor::fetch_all"
        - "QueryExecutor::execute"
      blocking_ctors:
        - "QueryExecutor::new (ALLOWLIST)"
        - "QueryExecutor::with_config (ALLOWLIST)"
      gaps:
        async_missing_blocking: []
        blocking_missing_async: []
      note: |
        The module-level `pub async fn execute_with_retry` is a
        free function, not an inherent method; the parity walker
        only collects `impl T { pub async fn ... }`. It is
        therefore out of the parity surface. If a future change
        needs a blocking sibling, it would be a free
        `pub fn execute_with_retry_blocking` in
        `blocking/executor.rs` — out of scope for this TD.

  # --- 8. Acceptance gate ------------------------------------------
  - path: VERIFY
    action: verify
    description: |
      All must pass in order, from a clean checkout:

        cargo check -p cclab-pg
        cargo test  -p cclab-pg --lib
        cargo test  -p cclab-pg --test test_async_blocking_parity
        cargo test  -p cclab-pg --test test_blocking_facade_shape
        cargo test  -p cclab-pg --test test_layer_boundaries

      `cargo check --workspace` is explicitly NOT part of the
      gate; the pre-existing score-cli template-path breakage is
      outside this scope.

      Negative gate: confirm `bulk.rs`, `executor.rs`, `row.rs`
      are NOT present in `NON_PARITY_FILES` after the change:

        ! grep -E '"(bulk|executor|row)\.rs"' \
            projects/pgkit/pg/tests/test_async_blocking_parity.rs

      EXEMPT lists discipline (R5): confirm no new entries in
      `EXEMPT_ASYNC_ONLY`, `EXEMPT_BLOCKING_ONLY`, or
      `ASYNC_SIDE_NON_IO` for the Row / BulkExecutor /
      QueryExecutor types. The carve-out is closed by pairing,
      not by exempting.
```

# Reviews

### Review 1
**Verdict:** approved

- [changes] The TD-time decisions are each resolved with a named choice plus rationale: Row mirrored as additional `impl Row` methods (no new struct — the parity walker pairs by short type name so the additional impl contributes its surface) with a `_blocking` name suffix to dodge Rust's "no two inherent fns with the same name" rule; BulkExecutor / QueryExecutor mirrored as new owning types under `blocking::*` because they carry per-instance state (config + pool borrow) that needs an `Arc<Runtime>` companion. The Row-suffix decision is reflected back into the parity walker (unit #5) via a one-line `strip_suffix("_blocking")` extension — minimal cost, no async-side rename.
- [changes] Unit ordering is dependency-correct: three new code files (#1 row, #2 bulk, #3 executor) → mod.rs wiring (#4) → parity carve-out narrowing + walker suffix-strip (#5) → ALLOWLIST extension (#6) → audit baseline (#7) → acceptance gate (#8). The blocking entries land BEFORE the test that gates on them, but neither the parity test nor the shape test can pass against an empty `blocking/` tree — and they don't have to: the test changes (#5, #6) are part of THIS change and together with #1-#4 form a single atomic landing.
- [changes] The R7 audit baseline is committed in-spec rather than left to the implementer — `audit_findings` enumerates every blocking entry by qualified name (`Row::insert_blocking`, etc.), names the count match (22-22, 3-3, 2-2), and explicitly notes the gaps lists are empty. The free-function `execute_with_retry` at module scope is called out as out-of-walker-scope (parity walks `impl T { ... }` only).
- [changes] Executor-generic collapse (R4) is correctly applied: the blocking facade fns take `&blocking::Connection` exclusively, never re-export `sqlx::Executor`, and pull the pool internally via `conn.as_async().pool()`. The TD also pre-flags the one signature risk (R6 — `F: ... + Clone` may need concession under sqlx lifetimes) with a documentation fallback rather than a code-change escape hatch.
- [changes] The hand-write protocol is correctly applied: all three new code files carry `HANDWRITE-BEGIN/HANDWRITE-END` markers naming the codegen gap (`facade-mirror` section type — closes when score grows a generator that emits `rt.block_on(async_fn(...))` wrappers from an async signature). `@spec` annotations point back to this TD section so future regenerators can locate the contract.
- [changes] Acceptance gate (unit #8) is mechanically checkable: per-crate `cargo check`, the two audit tests, lib tests, and `test_layer_boundaries` — plus two negative-gate `grep`s (no `bulk.rs|executor.rs|row.rs` in `NON_PARITY_FILES`; no new EXEMPT entries for the target types). `cargo check --workspace` is explicitly excluded per issue R8, justified by pre-existing score-cli template-path breakage outside this scope.

