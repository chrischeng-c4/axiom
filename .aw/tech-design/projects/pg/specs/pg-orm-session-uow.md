---
id: pg-orm-session-uow
fill_sections: [changes]
---

## Changes
<!-- type: changes lang: yaml -->

```yaml
# Lands `cclab_pg::orm::Session` (async-first) plus its
# `blocking::Session` sibling and the `cclab-pg-mamba` binding
# surface that exposes the Session machinery to mamba scripts. The
# Session brings two pieces of state to the existing driver layer:
#
#   identity map — per-Session HashMap<(table, pk), Arc<dyn Any>>.
#     Repeat `session.get::<M>(pk)` within one Session returns the
#     same Arc pointer (`Arc::ptr_eq` is true) — the cclab
#     translation of SQLAlchemy's `is`-identity guarantee. Map is
#     never global; `rollback` clears it. Type-erased internally so
#     a single Session can manage instances of many model types
#     without per-type maps.
#
#   unit-of-work staging — per-Session Vec<UowEntry> where UowEntry
#     ∈ {Insert(table, values, dirty_ref), Update(table, pk,
#     values, dirty_ref), Delete(table, pk)}. Stage entries do not
#     emit SQL until `flush` / `commit`. `flush` drains in
#     canonical order: all INSERTs first, then UPDATEs, then
#     DELETEs, intra-class staging order preserved.
#
# TD-time decisions resolved here:
#
#   Session type shape: single non-generic `Session<'a>` struct
#     borrowing `&'a Connection`. Per-method type parameters
#     (`session.get::<M>(pk)`, `session.add::<M>(model)`) carry the
#     model type. The identity map stores `Arc<dyn Any + Send +
#     Sync>` and downcasts on get. The alternative — `Session<M>`
#     per-type — was rejected because a real session manages
#     instances of MANY model types simultaneously (User + Post +
#     Tag in one transaction), so per-type sessions would force
#     either multiple Sessions per transaction or a HKT-shaped
#     macro layer. Type-erased map is simpler and matches SA's
#     internal design.
#
#   SessionModel trait: sealed via `orm::session::sealed::Sealed`
#     super-trait. v1 requires the five methods listed in issue R7
#     (`table`, `pk_column`, `pk`, `to_values`, `from_row`). No
#     derive in v1 — #2089 adds the derive that auto-impls
#     SessionModel from a `#[derive(Model)]` declarative class.
#     Manual impls in the integration test demonstrate the shape.
#
#   Transaction lifecycle: Session owns
#     `Option<sqlx::Transaction<'a, Postgres>>`. `begin()` opens
#     and stores Some(tx). `commit()` = drain UoW via `flush`,
#     then `tx.commit().await`, then `staging.clear()`. `rollback()`
#     = `tx.rollback().await`, then `staging.clear() +
#     identity_map.clear()`. If `flush`/`commit` is called without
#     an explicit `begin`, the operation is auto-wrapped in a
#     short-lived transaction opened just for that call.
#
#   flush ordering — three-pass drain. Pass 1 emits all
#     INSERTs in staging-order via `Row::insert(pool, table,
#     values)`; each result is downcast back to the original M and
#     the in-memory Arc is updated with the server-assigned pk
#     (RETURNING). Pass 2 emits UPDATEs via `Row::update(pool,
#     table, pk, values)`. Pass 3 emits DELETEs via `Row::delete(
#     pool, table, pk)`. Inside an outer transaction, the `pool`
#     param is replaced by `&mut **tx` (sqlx Transaction derefs to
#     Connection — which implements Executor). The blocking
#     sibling runs the identical pass-1/2/3 drain inside one
#     `rt.block_on(self.inner.flush())` call.
#
#   query::<M>() integration: returns
#     `SessionQuery<'a, 'sess, M>`, a small wrapper holding
#     `{ session: &'sess mut Session<'a>, builder: QueryBuilder,
#     _marker: PhantomData<M> }`. Terminators (`first`, `all`)
#     materialise rows via `M::from_row` and register each result
#     in the identity map keyed by `(M::table(), m.pk())`.
#     Subsequent `session.get::<M>(pk)` for the same pk returns
#     the same Arc (`ptr_eq` holds). Direct use of `QueryBuilder`
#     outside of session.query() does NOT participate in the
#     identity map — only the session-routed path registers.
#
#   Dirty tracking: NO setattr-style attribute interception in
#     v1 (that's a Python-style hook the Rust layer can't cheaply
#     replicate). Users call `session.touch(arc_clone)` to mark an
#     instance dirty for UPDATE on next flush. The mamba binding
#     exposes `session.touch(table_name, dict)` which routes
#     through the same Rust touch path. Issue R3's "per-model
#     dirty mark" parenthetical resolves here: explicit `touch`
#     in v1; declarative-model #2089 may add proxy-based
#     auto-dirty when it lands.
#
#   blocking::Session shape: NEW owning struct `blocking::Session<
#     'a> { inner: orm::Session<'a>, rt: Arc<Runtime> }`. Every
#     pub fn on the async `Session` has a sibling on the blocking
#     `Session` whose body is exactly
#     `self.rt.block_on(self.inner.<fn>(...))` — no control flow,
#     per the shape audit. The lifetime parameter survives the
#     facade boundary: blocking Session borrows from
#     `&'a blocking::Connection`, async-inner borrows from the
#     pool acquired through `conn.as_async()`.
#
#   Crate-root re-exports: `cclab_pg::Session` resolves to
#     `cclab_pg::orm::Session`; `cclab_pg::blocking::Session`
#     resolves to `cclab_pg::orm::blocking::Session`. This matches
#     the existing pattern for `Row`, `Connection`, `Transaction`
#     where the orm-module surface gets re-exported at the crate
#     root for ergonomics. The `cclab_pg::blocking` glob already
#     re-exports `crate::driver::blocking::*`; we add an explicit
#     `pub use crate::orm::session::blocking::Session;` line on
#     that module (or move `blocking` from `driver/blocking/` to
#     `crate::blocking` if needed — TD permits whichever ends up
#     cleaner; this is internal arrangement).
#
#   mamba binding surface (untyped): the mamba layer exposes
#     untyped Session operations keyed by table name and dict
#     payload. Generic `Session::add::<M>(model)` becomes
#     `mb_pg_session_add(session, table_name, dict)` on the mamba
#     side — the dict is converted to `HashMap<String,
#     ExtractedValue>` and stored on a per-table "anonymous M"
#     internally (a thin `RowModel { table, pk, values }` newtype
#     impls SessionModel for this purpose). The typed Rust surface
#     remains generic; mamba's narrowed surface only exercises the
#     RowModel impl. When #2089 ships, the mamba binding gets
#     additional typed methods for registered model classes; the
#     untyped surface stays for ad-hoc table access.
#
#   mamba verb count: 12 new FFI verbs on `MbPgSession`. Listed
#     in unit #6. The `methods.rs` doc table grows the same way
#     it grew for the Transaction surface (#2078 pattern).
#
#   Audit baseline (R8): the parity walker matches `Session` on
#     both sides automatically. New ALLOWLIST entries are
#     `Session::new` (constructor — borrows a Connection, no IO),
#     `Session::connection` (getter — returns the underlying
#     Connection ref). The shape audit verifies every other pub
#     fn on `blocking::Session` contains a `block_on` call. No
#     NON_PARITY_FILES additions; the carve-out residual stays at
#     the three migrate auxiliaries.
#
#   Hand-write protocol: ALL files in this change carry
#     `HANDWRITE-BEGIN` markers because the codegen gaps that
#     would generate them (orm-session section type, identity-map
#     state-machine emitter, mamba-FFI generator) do not exist
#     yet. Each marker names the gap. `@spec` annotations point
#     back to this TD section so future regenerators can locate
#     the contract.
#
#   Acceptance gate scope (issue R10): per-crate `cargo check`,
#     the two audit tests, lib tests, layer-boundary test, the
#     new identity-map integration test (skip-on-no-DATABASE_URL),
#     and the new pg-mamba binding test. Workspace-wide
#     `cargo check --workspace` is explicitly NOT in the gate
#     (pre-existing score-cli template-path breakage).

changes:
  # --- 1. orm::session module — async surface --------------------
  - path: projects/pgkit/pg/src/orm/session/mod.rs
    action: create
    description: |
      The async `Session` lives here. Three public items:

        pub struct Session<'a> {
            conn: &'a Connection,
            tx: Option<sqlx::Transaction<'a, sqlx::Postgres>>,
            identity_map: HashMap<(String, i64), Arc<dyn Any + Send + Sync>>,
            staging: Vec<UowEntry>,
        }

        pub trait SessionModel: sealed::Sealed + Any + Send + Sync + 'static {
            fn table() -> &'static str;
            fn pk_column() -> &'static str { "id" }
            fn pk(&self) -> i64;
            fn to_values(&self) -> Vec<(String, ExtractedValue)>;
            fn from_row(row: &Row) -> Result<Self> where Self: Sized;
        }

        pub struct SessionQuery<'a, 'sess, M: SessionModel> { ... }

      Internal `enum UowEntry { Insert { table, values, slot:
      Weak<dyn Any+Send+Sync> }, Update { table, pk, values }, Delete
      { table, pk } }`. The `slot: Weak` is what `flush` updates
      with the RETURNING pk after the INSERT lands. Sealed
      sub-module `sealed::Sealed` makes the trait impl-able only
      from inside the crate (sealing extension; #2089's derive
      will land its own `impl SessionModel` blocks inside
      cclab-pg's orm module).

      Method set (every one is `pub async fn` unless noted):

        impl<'a> Session<'a> {
            pub fn new(conn: &'a Connection) -> Self;            // sync ctor
            pub fn connection(&self) -> &Connection;             // sync getter
            pub async fn begin(&mut self) -> Result<()>;
            pub async fn commit(&mut self) -> Result<()>;
            pub async fn rollback(&mut self) -> Result<()>;
            pub async fn flush(&mut self) -> Result<()>;
            pub fn add<M: SessionModel>(&mut self, model: Arc<M>);   // sync stage
            pub fn delete<M: SessionModel>(&mut self, pk: i64);      // sync stage
            pub fn touch<M: SessionModel>(&mut self, model: Arc<M>); // sync stage
            pub async fn get<M: SessionModel>(&mut self, pk: i64) -> Result<Option<Arc<M>>>;
            pub fn query<M: SessionModel>(&mut self) -> SessionQuery<'a, '_, M>;
        }

      Note that `add` / `delete` / `touch` are sync because they
      only mutate the staging Vec; SQL is emitted only on `flush`
      / `commit`. The blocking sibling (unit #2) therefore does
      NOT need to `block_on` for these three — they are direct
      pass-throughs of the sync staging method. The parity walker
      will pair them by short name without trouble (both are
      sync on both sides; the walker pairs sync+sync as well as
      async+async via the canonical name).

      // HANDWRITE-BEGIN reason: orm-session machinery; no
      //   section type today emits identity-map + UoW state from
      //   a spec. Closes when score grows an `orm-session`
      //   section type that emits the Session struct + trait +
      //   flush state machine from a structured definition.
      //! @spec
      //!   .aw/tech-design/projects/pg/specs/pg-orm-session-uow.md#changes
      // HANDWRITE-END

  # --- 2. orm::session::blocking — blocking sibling --------------
  - path: projects/pgkit/pg/src/orm/session/blocking.rs
    action: create
    description: |
      Blocking facade mirror over unit #1. Owning struct:

        pub struct Session<'a> {
            inner: crate::orm::Session<'a>,
            rt: Arc<Runtime>,
        }

      Constructor borrows the runtime from a `blocking::Connection`:

        impl<'a> Session<'a> {
            pub fn new(conn: &'a blocking::Connection) -> Self {
                Self {
                    inner: crate::orm::Session::new(conn.as_async()),
                    rt: conn.runtime(),
                }
            }
        }

      Every async fn on the inner Session gets a sync sibling
      whose body is exactly `self.rt.block_on(self.inner.<fn>(
      ...))` — no control flow, no `match`/`if`, per the shape
      audit. The sync staging methods (`add`, `delete`, `touch`)
      are direct pass-throughs: `pub fn add<M>(...) { self.inner.
      add::<M>(...) }` — no `block_on` (they don't await
      anything). The shape audit's BlockOnFinder walks the body
      for `block_on`; for these three methods, the ALLOWLIST
      entry covers the no-block_on case (constructors/getters/
      borrow helpers in the audit's own language — extended to
      "sync staging methods that don't await anything").

      `query::<M>` returns `SessionQuery` of the BLOCKING flavour
      — a sibling wrapper whose terminators internally `block_on`.

      // HANDWRITE-BEGIN reason: facade-mirror codegen gap —
      //   same as #2086. Closes when score grows a `facade-mirror`
      //   section type.
      //! @spec
      //!   .aw/tech-design/projects/pg/specs/pg-orm-session-uow.md#changes
      // HANDWRITE-END

  # --- 3. orm/mod.rs — wire session in -------------------------
  - path: projects/pgkit/pg/src/orm/mod.rs
    action: update
    description: |
      Add module declaration + re-export at the top of the file:

        pub mod session;
        pub use session::{Session, SessionModel, SessionQuery};

        pub mod blocking {
            pub use crate::orm::session::blocking::{Session, SessionQuery};
        }

      The `pub mod blocking` within `orm/` provides
      `cclab_pg::orm::blocking::Session` as the canonical path.
      The crate root `lib.rs` glob re-exports already lift orm
      symbols to crate root, so `cclab_pg::Session` and
      `cclab_pg::orm::Session` both resolve to the same type.

      For `cclab_pg::blocking::Session`: add a single line to
      `crate::driver::blocking::mod.rs` (or a sibling re-export
      module) that re-exports the orm blocking Session under the
      crate::blocking namespace. Concretely:

        // in driver/blocking/mod.rs (or a new src/blocking.rs)
        pub use crate::orm::blocking::{Session, SessionQuery};

      This is a deliberate cross-module re-export — `driver::
      blocking::Session` is the orm Session, not a driver-layer
      type. The layer-boundary test continues to enforce one-way
      coupling (orm uses driver, never the reverse) because the
      re-export is `pub use` only; no `orm` symbol is depended on
      from within `driver/` business logic.

  # --- 4. Integration test for Session semantics ----------------
  - path: projects/pgkit/pg/tests/test_session_identity_map.rs
    action: create
    description: |
      Integration test covering Session/UoW/identity-map against
      a real Postgres database. Skip-on-missing-DATABASE_URL
      following the existing integration-test pattern.

      Test fixture: a `TestUser { id, name, email }` struct with
      a hand-written `impl SessionModel for TestUser`. Schema
      bootstrap creates a `test_users` table at fixture time and
      drops it at teardown.

      Tests:

        identity_map_repeat_get_returns_same_arc
            Seed one row. Call `s.get::<TestUser>(1)` twice.
            Assert `Arc::ptr_eq(&first, &second)`.

        staging_order_preserved_through_flush
            `s.add(u_a); s.add(u_b); s.flush()`. Assert DB rows
            were inserted in (u_a, u_b) order by inspecting
            inserted_at or the order returned from a
            SELECT * ORDER BY id.

        flush_canonical_order_insert_update_delete
            Stage one of each. After flush, assert SQL log (via
            sqlx's QueryLogger or a counted wrapper around
            execute) shows INSERTs before UPDATEs before DELETEs.

        commit_clears_staging_keeps_identity_map
            Stage + commit. Assert `staging.is_empty()` (via a
            test-only accessor) and `s.get::<TestUser>(pk)`
            returns Some with the cached Arc.

        rollback_clears_both_staging_and_identity_map
            Stage + rollback. Assert both maps empty;
            subsequent `s.get::<TestUser>(pk)` re-fetches from
            DB (round-trip count goes up by 1).

        query_routes_through_identity_map
            `s.query::<TestUser>().filter(...).all()`. Assert
            that subsequent `s.get::<TestUser>(returned_pk)`
            returns the SAME Arc as the one from the query.

      // HANDWRITE-BEGIN reason: integration tests are
      //   inherently hand-written until codegen learns a
      //   `test-plan-integration` section type.
      //! @spec
      //!   .aw/tech-design/projects/pg/specs/pg-orm-session-uow.md#changes
      // HANDWRITE-END

  # --- 5. Layer-boundary baseline -------------------------------
  - path: projects/pgkit/pg/tests/test_layer_boundaries.rs
    action: update
    description: |
      No code change required if the existing test already
      enforces one-way orm → driver coupling without naming
      the session submodule explicitly. If the existing test
      enumerates orm submodules, add `session` to the list of
      orm-internal modules that may import from `driver::*`.

      Negative gate: verify `grep -r 'use cclab_pg::orm' projects/
      pgkit/pg/src/driver/` returns zero results after this
      change lands.

  # --- 6. pg-mamba binding — MbPgSession + 12 FFI verbs -------
  - path: projects/pgkit/mamba-binding/src/types.rs
    action: update
    description: |
      Add a new opaque handle alongside the existing
      `MbPgConnection`, `MbPgTransaction`, `MbPgMigrationRunner`,
      `MbPgMigration`:

        pub struct MbPgSession {
            // Wraps a blocking::Session bound to a Connection
            // held in this very handle. The Session lifetime is
            // tied to the connection's lifetime via an
            // Arc<PgConnection> kept alive inside the handle.
            pub inner: Mutex<Option<OwnedSession>>,
        }

      `OwnedSession` is a small private wrapper (defined in the
      same file or in `src/session.rs`) that holds both
      `Arc<PgConnection>` and a `'static`-lifetime-trick
      `cclab_pg::blocking::Session` (via a self-referential
      struct, or via `Arc<PgConnection>` plus a runtime-scoped
      executor borrow — implementation chooses the simpler
      route). The `Mutex<Option<...>>` lets `commit` / `rollback`
      consume the inner session in place (matching the
      `MbPgTransaction` pattern from #2078).

      Update the module's doc table to add the 12 new verbs.

  - path: projects/pgkit/mamba-binding/src/session.rs
    action: create
    description: |
      New file alongside `methods.rs`. Contains the 12 FFI verbs
      for the Session surface, plus the OwnedSession lifetime
      wrapper if not placed in `types.rs`.

      The 12 verbs (paralleling Connection's 4 + Transaction's 7
      + MigrationRunner's 9 from #2078):

        mb_pg_session_new(conn) -> session
        mb_pg_session_begin(session)
        mb_pg_session_commit(session)
        mb_pg_session_rollback(session)
        mb_pg_session_flush(session)
        mb_pg_session_add(session, table, dict)
        mb_pg_session_delete(session, table, pk)
        mb_pg_session_touch(session, table, dict)
        mb_pg_session_get(session, table, pk) -> Option<dict>
        mb_pg_session_query_all(session, table, filter_dict) -> list<dict>
        mb_pg_session_query_first(session, table, filter_dict) -> Option<dict>
        mb_pg_session_close(session)

      Each verb is `extern "C" fn name(args: *const MbValue,
      nargs: usize) -> MbValue` and follows the exact ABI shape
      of the existing `mb_pg_*` verbs in `methods.rs`.

      Internal dispatch on the typed Rust Session: a
      `RowModel { table: String, pk: i64, values: HashMap<
      String, ExtractedValue> }` newtype implements SessionModel
      with `table() -> "<dynamic>"` returning a sentinel and the
      actual table threaded through the staging entry directly.

      Wait — SessionModel::table() is `&'static str`, which
      can't carry a dynamic per-instance table name. Resolution:
      add a second, internal trait `DynSessionModel` with `fn
      table(&self) -> &str` (instance-level) that the typed
      `Session::add` / `Session::delete` / etc. fall back to
      when M = RowModel. Or simpler: the Rust Session keeps the
      typed surface AND also exposes a `add_dyn(table: &str,
      values: ...)` family that the mamba binding uses
      directly, bypassing the SessionModel trait for the
      untyped case. The TD's typed surface from unit #1 stays;
      the dyn surface is internal to orm::session and used only
      by the mamba binding glue.

      Resolved decision: unit #1's Session also exposes
      a small parallel set of dyn methods:

        pub fn add_dyn(&mut self, table: &str, values: Vec<(String, ExtractedValue)>);
        pub fn delete_dyn(&mut self, table: &str, pk: i64);
        pub fn touch_dyn(&mut self, table: &str, pk: i64, values: Vec<(String, ExtractedValue)>);
        pub async fn get_dyn(&mut self, table: &str, pk: i64) -> Result<Option<HashMap<String, ExtractedValue>>>;
        pub async fn query_all_dyn(&mut self, table: &str, filter: &[(String, ExtractedValue)]) -> Result<Vec<HashMap<...>>>;
        pub async fn query_first_dyn(&mut self, table: &str, filter: &[(...)]) -> Result<Option<HashMap<...>>>;

      The dyn methods share storage with the typed methods —
      both stage into the same UowEntry vector. The mamba
      binding routes to `_dyn` exclusively. Typed Rust users
      use the generic `<M>` methods.

      // HANDWRITE-BEGIN reason: mamba-FFI generator codegen
      //   gap; no section type today emits MbValue ABI
      //   wrappers around a typed Rust API. Closes when score
      //   grows a `mamba-binding` section type.
      //! @spec
      //!   .aw/tech-design/projects/pg/specs/pg-orm-session-uow.md#changes
      // HANDWRITE-END

  - path: projects/pgkit/mamba-binding/src/lib.rs
    action: update
    description: |
      Wire `pub mod session;` into the crate root alongside
      `pub mod methods;`, `pub mod types;`, `pub mod lib_migrate;`.

      Update the `linkme` distributed slice (if used for
      mamba-registry FFI registration) so the 12 new
      `mb_pg_session_*` symbols are registered at crate load.

  # --- 7. pg-mamba binding test --------------------------------
  - path: projects/pgkit/mamba-binding/tests/test_session_binding.rs
    action: create
    description: |
      Mamba binding integration test. Skip-on-missing-DATABASE_URL.
      Tests cover the 12 FFI verbs end-to-end through the MbValue
      ABI (same shape as `test_blocking_binding.rs`):

        session_lifecycle_new_close
            Open connection, create session, close it.

        session_add_commit_then_get_round_trips
            add(test_users, {name: "alice"}), commit, get(test_users,
            returned_pk) -> dict with name="alice".

        session_rollback_discards_staging_and_clears_identity_map
            add, rollback, get -> None (no pk yet because INSERT
            was rolled back).

        session_query_all_returns_dicts
            Seed two rows, query_all(test_users, {}) -> list of two
            dicts.

      // HANDWRITE-BEGIN reason: same as unit #4 — integration
      //   tests are hand-written until a test-plan-integration
      //   section type lands.
      //! @spec
      //!   .aw/tech-design/projects/pg/specs/pg-orm-session-uow.md#changes
      // HANDWRITE-END

  # --- 8. Audit baseline ---------------------------------------
  - path: projects/pgkit/pg/tests/test_blocking_facade_shape.rs
    action: update
    description: |
      Extend the ALLOWLIST with the Session entries that are
      sync-only (constructors, getters, sync staging methods
      that don't await):

        ("Session", "new",         "constructor: wraps async Session + clones runtime (no IO)"),
        ("Session", "connection",  "getter: borrows the underlying blocking Connection"),
        ("Session", "add",         "sync staging: mutates UoW Vec only, no IO"),
        ("Session", "delete",      "sync staging: mutates UoW Vec only, no IO"),
        ("Session", "touch",       "sync staging: mutates UoW Vec only, no IO"),
        ("Session", "query",       "sync builder ctor: returns SessionQuery, no IO"),
        ("Session", "add_dyn",     "sync staging dyn variant: mutates UoW Vec only, no IO"),
        ("Session", "delete_dyn",  "sync staging dyn variant: mutates UoW Vec only, no IO"),
        ("Session", "touch_dyn",   "sync staging dyn variant: mutates UoW Vec only, no IO"),

      The blocking-side `Session::begin`, `commit`, `rollback`,
      `flush`, `get`, `get_dyn`, `query_all_dyn`, `query_first_dyn`
      are NOT on the ALLOWLIST — their bodies must contain
      `block_on` (the shape audit's BlockOnFinder verifies this).

  - path: projects/pgkit/pg/tests/test_async_blocking_parity.rs
    action: gate
    description: |
      No edit required. The parity walker matches the new
      `Session` (TypeName) pair automatically:
      `orm::Session` (async) vs `orm::blocking::Session` (sync)
      both have the same short name `Session` and the walker
      pairs by short name across the async + blocking subtrees.

      Verify: the walker emits zero `EXEMPT_*` warnings for
      Session; the `NON_PARITY_FILES` carve-out list is unchanged
      (still the three migrate auxiliaries).

      Negative gate: `grep -E 'session(\.|/)' projects/pgkit/pg/
      tests/test_async_blocking_parity.rs` returns no NEW
      NON_PARITY_FILES additions (matches existing migrate-only
      pattern).

  # --- 9. Acceptance gate (issue R10) --------------------------
  - path: ACCEPTANCE_GATE
    action: gate
    description: |
      All from a clean checkout against the post-change tree:

        cargo check -p cclab-pg
        cargo check -p cclab-pg-mamba
        cargo test  -p cclab-pg --lib
        cargo test  -p cclab-pg --test test_async_blocking_parity
        cargo test  -p cclab-pg --test test_blocking_facade_shape
        cargo test  -p cclab-pg --test test_layer_boundaries
        cargo test  -p cclab-pg --test test_session_identity_map      # NEW; skips without DATABASE_URL
        cargo test  -p cclab-pg-mamba --test test_blocking_binding
        cargo test  -p cclab-pg-mamba --test test_layer_boundaries
        cargo test  -p cclab-pg-mamba --test test_session_binding     # NEW; skips without DATABASE_URL

      Workspace-wide `cargo check --workspace` is explicitly NOT
      in the gate (pre-existing score-cli template-path
      breakage stays out of this scope).

audit_findings:
  - [changes] Internal split is two new files (`orm/session/mod.rs` +
    `orm/session/blocking.rs`) plus one new file in pg-mamba
    (`session.rs`). The orm split is justified by the parity
    invariant: the blocking facade is structurally a separate file
    so the audit can locate and walk it. Three sub-files is the
    minimum split — putting blocking inside `mod.rs` would
    conflate async + blocking code under one syn::File, breaking
    the audit walker's per-subtree pairing.
  - [changes] Type-erased identity map (`HashMap<(String, i64),
    Arc<dyn Any + Send + Sync>>`) was chosen over per-type maps
    because a real Session manages instances of many model types
    simultaneously. Downcast cost on `get` is O(1) — a single
    `Arc::downcast::<M>` per lookup. The downcast can fail iff the
    caller stores instance of type A and looks up type B at the
    same pk — that's a user error and surfaces as
    `Err(SessionError::IdentityMapTypeMismatch)`.
  - [changes] The `_dyn` parallel surface (`add_dyn`, `get_dyn`,
    etc.) is a deliberate carve-out for the mamba binding. The
    typed `<M: SessionModel>` surface is the canonical Rust API;
    `_dyn` exists only because mamba dispatches by table-name
    string rather than Rust type. Both surfaces share the UoW
    staging vector — the only difference is whether the staging
    entry carries a typed `Arc<dyn Any>` (typed path) or a
    HashMap (dyn path). flush() handles both uniformly.
  - [changes] Lifetime parameter survives the facade boundary
    (`Session<'a>` on both sides). The blocking sibling holds
    `inner: orm::Session<'a>` so the borrow chain `&'a
    blocking::Connection → &'a Connection (inner) → &'a Session
    (typed)` stays explicit. Avoids the `'static` self-referential
    struct hazard that would otherwise creep in.
  - [changes] flush ordering (INSERT → UPDATE → DELETE) is the
    SQLAlchemy canonical order. Justification: DELETEs after
    UPDATEs preserves the case where a user updates an instance
    then deletes it in the same session — the UPDATE round-trip
    happens first (no-op effectively, but correct ordering) and
    then the DELETE removes the row. The alternative — eliding
    UPDATEs on instances queued for DELETE — is a v2 optimisation;
    v1 emits both for simplicity.
  - [changes] Mamba surface is 12 verbs (vs Connection's 4,
    Transaction's 7, MigrationRunner's 9 from #2078). Total
    pg-mamba FFI surface grows from 17 → 29 verbs.
  - [changes] All units carry HANDWRITE markers naming three
    distinct codegen gaps: `orm-session` (the Session struct +
    trait + flush state machine), `facade-mirror` (the blocking
    sibling, same gap as #2086), `mamba-binding` (the MbValue
    ABI wrappers). Each gap is independently closable in score
    as a new section type; until then, hand-write is the
    documented temp state.
  - [changes] Acceptance gate is mechanically checkable: 10
    cargo invocations, no manual inspection required. The two
    new test files skip gracefully without DATABASE_URL so the
    gate passes in CI environments without Postgres (matching
    existing integration-test convention).
```

# Reviews

### Review 1
**Verdict:** approved

- [changes] Nine-unit decomposition cleanly carves the vertical (Rust `orm::session::{mod,blocking}` + wiring + integration tests + pg-mamba binding) and lands the SA parity gap atomically. The audit-baseline unit (#8 — `test_blocking_facade_shape.rs` ALLOWLIST extensions for `Session::new` / `Session::connection`) plus the #9 acceptance gate covering 10 cargo invocations gives reviewers a single green/red signal.
- [changes] HANDWRITE markers on every unit enumerate the three codegen gaps explicitly — `orm-session` section type (identity-map + UoW state machine emitter), `facade-mirror` section type (blocking siblings from async signatures, the same gap #2086 left open), and `mamba-binding` section type (the 12 FFI verbs from a typed surface). Each `@spec` annotation back-references this TD so a future regenerator can locate the contract — meets the HANDWRITE-protocol requirement.
- [changes] Type-erased identity map (single non-generic `Session<'a>` storing `Arc<dyn Any + Send + Sync>`, downcast on `get::<M>`) correctly handles the multi-type Session case (User + Post + Tag in one transaction) that per-type sessions cannot serve. Sealed `SessionModel` trait keeps the v1 surface forward-compatible with #2089's `#[derive(Model)]` auto-impl.
- [changes] Dual typed/`_dyn` surface for the mamba binding (Rust callers use generic `add::<M>`/`get::<M>`; mamba routes through `add_dyn(table, dict)`/`get_dyn(table, pk)` over the internal `RowModel` newtype) is the right resolution for the Rust-generic-vs-mamba-untyped tension. Both paths share the same UoW staging vector so flush ordering INSERT → UPDATE → DELETE is invariant of caller.
- [changes] Flush-ordering three-pass drain reuses `Row::insert` / `Row::update` / `Row::delete` (with `&mut **tx` substituted for the pool inside an outer transaction) instead of introducing a parallel write path — keeps the orm layer thin and the parity walker happy: each driver call already has its blocking sibling, no new parity entries needed.

