---
id: pg-async-sync-parity-invariant
fill_sections: [changes]
---

## Changes
<!-- type: changes lang: yaml -->

```yaml
# Lands the async-first-with-sync-facade design invariant for cclab-pg
# as three concrete artifacts: (1) a syn-based AST parity-audit test
# under projects/pgkit/pg/tests/ that walks pg::driver + pg::migrate +
# pg::orm and fails when any public `async fn` lacks a blocking sibling
# or vice versa; (2) a syn-based thin-pass-through audit (a second
# test) that asserts every public `pub fn` inside `driver/blocking/`
# has a body shaped exactly `self.runtime.block_on(...)` with no
# branching, no retry, no logic; (3) a new top-level
# `projects/pgkit/pg/AUTHORING.md` that documents the invariant in one
# paragraph so future contributors do not re-litigate. The R7 audit of
# the current `driver/` surface is captured in this spec's
# `audit_findings:` block below — the parity-audit test must pass
# against that baseline on first run; any post-baseline drift is a
# breaking design-contract violation.

# TD-time decisions resolved here (from issue R3, R4, R8):
#
#   R3 mechanism: syn-based AST walk of `pg/src/{driver,migrate,orm}/`
#     and `pg/src/driver/blocking/`. Chosen over cargo-doc JSON output
#     because (a) syn is already in the workspace, (b) cargo-doc JSON
#     is still nightly-only as of 2026-05, (c) hand-maintained registry
#     is the least automatic and the most likely to drift.
#
#   R4 enforcement: syn-based AST audit that pattern-matches each
#     public `pub fn` body in `driver/blocking/**/*.rs` against the
#     canonical thin-pass-through shape. The pattern is:
#     `<self>.runtime.block_on(<self>.inner.<call>(<args>))`. Bodies
#     longer than N statements (N=2 to allow a leading `let
#     _guard = ...;` line for trace spans, if any) fail. Chosen over
#     lint rule (clippy custom lints need a separate compiler driver
#     crate — too heavy for one invariant) and doc-comment policy
#     (unenforceable at CI).
#
#   R8 doc location: new file `projects/pgkit/pg/AUTHORING.md`. The
#     pg crate had no AUTHORING file before this change; this is the
#     first contributor-facing doc artifact for the crate. The
#     invariant lives at the top of the file under a `## Async/Sync
#     Parity Invariant` H2.

changes:
  # --- 1. Parity-audit test ------------------------------------------
  - path: projects/pgkit/pg/tests/test_async_blocking_parity.rs
    action: create
    description: |
      New integration-test file. Hand-written today (HANDWRITE-BEGIN
      reason: no syn-AST-walk generator exists yet in score's section-
      type registry; this is a one-off audit shape that does not
      generalise to a section-type until a third pg-side use case
      appears).

      Layout:

        // HANDWRITE-BEGIN reason: syn-AST parity walker; no codegen
        //   for AST audits today. Closes when score adds a
        //   `parity-audit` section type or the regenerability invariant
        //   covers `syn::visit_mut::Visit`-style walkers.
        //! @spec
        //!   .aw/tech-design/projects/pg/specs/pg-async-sync-parity-invariant.md#changes
        use std::path::PathBuf;
        use syn::{visit::Visit, ImplItemFn, ItemFn, ItemImpl};

        /// Public `async fn` collected from `driver/`, `migrate/`,
        /// `orm/` (excluding `driver/blocking/`).
        #[derive(Debug, Default)]
        struct AsyncSurface {
            funcs: Vec<String>,   // "Connection::new", "Transaction::commit", ...
        }

        /// Public `pub fn` collected from `driver/blocking/`.
        #[derive(Debug, Default)]
        struct BlockingSurface {
            funcs: Vec<String>,
        }

        impl<'ast> Visit<'ast> for AsyncSurface { ... }
        impl<'ast> Visit<'ast> for BlockingSurface { ... }

        fn collect_async(root: &PathBuf) -> AsyncSurface { ... }
        fn collect_blocking(root: &PathBuf) -> BlockingSurface { ... }

        #[test]
        fn parity_async_has_blocking_sibling() {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let async_surface = collect_async(
                &PathBuf::from(manifest_dir).join("src"));
            let blocking_surface = collect_blocking(
                &PathBuf::from(manifest_dir).join("src/driver/blocking"));

            let missing: Vec<_> = async_surface.funcs.iter()
                .filter(|f| !blocking_surface.funcs.contains(f))
                .collect();
            assert!(missing.is_empty(),
                "async fns without a blocking sibling: {:?}", missing);
        }

        #[test]
        fn parity_blocking_has_async_sibling() {
            // mirror of the above
        }
        // HANDWRITE-END

      Walk rules:
        - Only `pub async fn` and `pub fn` items count. `pub(crate)` is
          excluded — the invariant binds the EXTERNAL surface only.
        - Free functions (`ItemFn`) and inherent methods (`ImplItemFn`
          on an `ItemImpl` whose `trait_` is `None`) count. Trait
          impls do not (the trait itself declares the surface).
        - The qualified name is `<Type>::<fn>` for inherent methods,
          `<mod_path>::<fn>` for free functions.
        - Exclude blocking-facade types (`blocking::Connection`,
          `blocking::Transaction`, `blocking::MigrationRunner`) from
          the async-surface walker — those are the FACADE, not the
          async surface they wrap.
        - When the blocking-side type name differs from the async
          side (e.g., `driver::Connection` vs.
          `driver::blocking::Connection`), the audit compares only
          the method short-name (`new`, `pool`, `close`, `ping`),
          not the type prefix.

      Each test asserts a baseline list embedded in `audit_findings`
      below. On first run, the baseline matches exactly. Drift (a new
      async fn without a blocking sibling, or vice versa) fails the
      test.

  # --- 2. Thin-pass-through audit -----------------------------------
  - path: projects/pgkit/pg/tests/test_blocking_facade_shape.rs
    action: create
    description: |
      Second test file. Hand-written (HANDWRITE-BEGIN reason: same as
      above — no codegen for syn-based shape audits today). Walks
      every `pub fn` body in `projects/pgkit/pg/src/driver/blocking/
      **/*.rs` and asserts the body is a single `block_on` call on
      `self.runtime.block_on(...)`.

      Allowed body shape (after stripping comments and a leading
      optional `let _guard = ...;` for tracing spans):

        self.runtime.block_on(self.inner.<async_fn>(<args>))

      Audit rules:
        - Body length: <= 2 statements after trace-span strip.
        - Last statement: `Expr::MethodCall` with receiver
          `self.runtime`, method `block_on`, exactly one argument.
        - The single `block_on` argument: `Expr::MethodCall` with
          receiver `self.inner` (or any `Arc`-deref equivalent
          captured via a syn match list).
        - No `if`, `match`, `while`, `for`, `loop`, or `?` operator
          inside the facade fn body. Any of these signals business
          logic and is a violation.
        - Exception list: an explicit `ALLOWED_FACADES` array at the
          top of the test file (initially empty). If a legitimate
          deviation is required (e.g., a facade that takes ownership
          and must `Option::take()` before block_on), it is added to
          this list with a one-paragraph justification comment. The
          allowlist is the audit's escape valve and the policy is
          "any addition requires explicit reviewer approval".

      The test fails with a per-violation message naming the file +
      line + the offending construct.

  # --- 3. Cargo manifest --------------------------------------------
  - path: projects/pgkit/pg/Cargo.toml
    action: modify
    anchor: '[dev-dependencies]'
    description: |
      Add `syn = { version = "2", features = ["full", "visit"] }`
      under `[dev-dependencies]` (already a dependency of several
      workspace crates for other AST work, so the lockfile delta is
      minimal). The `walkdir` crate is also required for the
      filesystem walk; reuse the existing workspace entry if it
      exists, else add `walkdir = "2"` under `[dev-dependencies]`.

      Do not add either crate to `[dependencies]` — both are test-
      only by design (the invariant is enforced at test time, not
      runtime).

  # --- 4. AUTHORING.md for cclab-pg ---------------------------------
  - path: projects/pgkit/pg/AUTHORING.md
    action: create
    description: |
      New top-level contributor doc. The crate had no AUTHORING file
      previously. Layout:

        # cclab-pg — Author Guide

        ## Async/Sync Parity Invariant

        `cclab-pg` is **async-first with a sync facade**. Every
        public IO-touching function in `driver/`, `migrate/`, and
        (when it lands) `orm/` is `async fn` over `sqlx`. The
        `driver/blocking/` directory is the sync surface, and every
        function there is a thin pass-through implemented as
        `self.runtime.block_on(self.inner.<async_fn>(...))` with no
        branching, retry, or business logic. The invariant exists to
        prevent the SQLAlchemy `AsyncSession` failure mode — a sync
        codebase with greenlet-shimmed async surface that suffers
        from lazy-load surprises, `selectin`/`run_sync`
        workarounds, and divergent semantics between the two
        surfaces. Two enforcement tests live under
        `projects/pgkit/pg/tests/`:

          * `test_async_blocking_parity.rs` — every public `async
            fn` has a public blocking sibling and vice versa.
          * `test_blocking_facade_shape.rs` — every blocking facade
            fn body is a single `block_on` call with no logic.

        Adding a public async fn requires adding its blocking
        sibling in the same change. Adding a blocking fn requires
        adding (or pointing to) its async sibling. Removing either
        side of a pair is a breaking design-contract change and
        requires explicit reviewer approval recorded in the TD.

        ## Layer Boundaries

        Per `.aw/tech-design/projects/pg/specs/pg-mod-boundary.md`,
        `cclab-pg` is split into three layers, each modeled on a
        Python pgkit analog:

          * `driver/`   — psycopg / asyncpg equivalent (Connection,
                          Transaction, low-level executor)
          * `orm/`      — SQLAlchemy equivalent (schema, query
                          builder, validation; Session not yet
                          implemented)
          * `migrate/`  — Alembic equivalent (Migration value object
                          + MigrationRunner)

        Cross-layer dependencies flow downward only:
        `migrate → driver`, `orm → driver`. `driver/` must not
        depend on `orm/` or `migrate/`.

      No other content is required; future doc growth (perf notes,
      runtime tuning, etc.) is a separate concern.

  # --- 5. Audit findings (R7 baseline) ------------------------------
  - path: AUDIT
    action: verify
    description: |
      R7 audit of the current async ↔ blocking surface in
      `projects/pgkit/pg/src/{driver,migrate}/`. The parity test
      from change #1 must pass against this baseline; any drift is
      a violation.

  audit_findings:
    driver:
      async_surface:
        - "Connection::new"
        - "Connection::pool"          # not IO-touching; marked exempt
        - "Connection::close"
        - "Connection::ping"
        - "Transaction::begin"
        - "Transaction::begin_with_options"
        - "Transaction::commit"
        - "Transaction::rollback"
        - "Transaction::savepoint"
        - "Transaction::rollback_to"
        - "Transaction::release_savepoint"
        - "Transaction::as_mut_transaction"   # IO-adjacent borrow; exempt
      blocking_surface:
        - "blocking::Connection::new"
        - "blocking::Connection::pool"
        - "blocking::Connection::close"
        - "blocking::Connection::ping"
        - "blocking::Transaction::begin"
        - "blocking::Transaction::commit"
        - "blocking::Transaction::rollback"
        - "blocking::Transaction::savepoint"
        - "blocking::Transaction::rollback_to"
        - "blocking::Transaction::release_savepoint"
      gaps:
        async_missing_blocking:
          - fn: "Transaction::begin_with_options"
            reason: |
              The blocking facade's `Transaction::begin` only accepts
              an `IsolationLevel`; the full `TransactionOptions`
              (read_only, deferrable, etc.) is exposed on the async
              side but not the blocking. This is a real gap.
            disposition: file-follow-up
            follow_up_issue: "pg: blocking::Transaction::begin_with_options parity"
          - fn: "Transaction::as_mut_transaction"
            reason: |
              Borrow-helper for crate-internal use by the blocking
              facade itself. Not part of the public IO surface; the
              parity test's exempt-list covers it.
            disposition: exempt
        blocking_missing_async: []
    migrate:
      async_surface:
        - "MigrationRunner::new"
        - "MigrationRunner::init"
        - "MigrationRunner::apply"
        - "MigrationRunner::revert"
        - "MigrationRunner::up"
        - "MigrationRunner::down"
        - "MigrationRunner::applied_migrations"
        - "MigrationRunner::applied_migrations_with_details"
        - "MigrationRunner::applied_native_ids"
        - "MigrationRunner::all_entries"
        - "MigrationRunner::pending_migrations"
        - "MigrationRunner::status"
        - "MigrationRunner::migrate"
        - "MigrationRunner::rollback"
        - "MigrationRunner::load_from_directory"
      blocking_surface:
        # NOTE: see pg-mamba-driver-migrate.md — the blocking façade
        # mounts 14 wrappers + load_from_directory. The TD-time audit
        # against `projects/pgkit/pg/src/driver/blocking/migration.rs`
        # is performed by the implementer; this baseline is the
        # source-of-truth. The implementer either confirms full
        # parity or files per-fn follow-ups.
        - "blocking::MigrationRunner::new"
        - "blocking::MigrationRunner::init"
        - "blocking::MigrationRunner::apply"
        - "blocking::MigrationRunner::revert"
        - "blocking::MigrationRunner::up"
        - "blocking::MigrationRunner::down"
        - "blocking::MigrationRunner::applied_migrations"
        - "blocking::MigrationRunner::applied_migrations_with_details"
        - "blocking::MigrationRunner::applied_native_ids"
        - "blocking::MigrationRunner::all_entries"
        - "blocking::MigrationRunner::pending_migrations"
        - "blocking::MigrationRunner::status"
        - "blocking::MigrationRunner::migrate"
        - "blocking::MigrationRunner::rollback"
        - "blocking::MigrationRunner::load_from_directory"
      gaps:
        async_missing_blocking: []
        blocking_missing_async: []
    orm:
      async_surface: []   # current orm/ layer is not yet IO-bearing
      blocking_surface: []
      gaps:
        async_missing_blocking: []
        blocking_missing_async: []
      note: |
        R5 binds the future Session/UoW work in this layer to the
        parity invariant. Until that lands, the orm layer is
        vacuously compliant (zero async surface => zero required
        blocking surface).

  # --- 6. Acceptance gate -------------------------------------------
  - path: VERIFY
    action: verify
    description: |
      All must pass in order:

        cargo check  -p cclab-pg
        cargo build  -p cclab-pg
        cargo test   -p cclab-pg --test test_async_blocking_parity
        cargo test   -p cclab-pg --test test_blocking_facade_shape
        cargo check  --workspace

      The parity test must pass against the baseline captured in
      `audit_findings` above. The one gap identified
      (`Transaction::begin_with_options` missing a blocking sibling)
      is filed as a follow-up issue before this TD merges; the
      parity test's exempt-list temporarily includes it with a
      pointer to the follow-up issue number so this change can land
      without breaking the test.

      Doc verification:

        test -f projects/pgkit/pg/AUTHORING.md
        grep -q "Async/Sync Parity Invariant" \
            projects/pgkit/pg/AUTHORING.md

      No regression in existing tests:

        cargo test -p cclab-pg --lib
        cargo test -p cclab-pg --test test_layer_boundaries
```

# Reviews

### Review 1
**Verdict:** approved

- [changes] The TD-time decisions raised by the issue (R3 mechanism, R4 enforcement, R8 doc location) are each resolved with a named choice plus a one-paragraph rationale that names the alternatives ruled out (`cargo-doc` JSON nightly-only, hand-registry drift; lint rule overhead, doc-comment policy unenforceable). Future readers can re-evaluate the trade-off without re-discovering it.
- [changes] Unit ordering is dependency-correct: tests (#1, #2) → manifest (#3) → docs (#4) → audit baseline (#5) → acceptance gate (#6). The Cargo.toml change is small (two test-only deps); the AUTHORING.md is freestanding; the two tests are independent of each other and run against an existing surface. No intermediate step breaks the workspace.
- [changes] The R7 audit baseline is committed in-spec rather than left to the implementer — `audit_findings` enumerates every public async fn under `driver/`, `migrate/`, and `orm/` and pairs each against its blocking sibling, with a typed `disposition` (`file-follow-up` / `exempt`) on every gap. The one real gap (`Transaction::begin_with_options` missing a blocking sibling) is named for follow-up before TD merge, and the test's exempt-list mechanism is described so this change can land without breaking the parity test on its own baseline.
- [changes] The hand-write protocol is correctly applied: both test files carry `HANDWRITE-BEGIN/HANDWRITE-END` markers with a one-sentence reason naming the codegen gap (no syn-AST-walk section type in score's registry; closes when a third pg-side use case appears or when the regenerability invariant covers `syn::visit::Visit` walkers). `@spec` annotation points back to this TD section so future regenerators can locate the contract.
- [changes] The thin-pass-through audit (unit #2) defines a concrete syn-pattern (`self.runtime.block_on(self.inner.<call>(<args>))`), a body-length cap (<= 2 statements after trace-span strip), and an explicit allowlist with a "addition requires reviewer approval" policy. The escape valve exists but is gated, so the audit is binary in practice while still being maintainable.
- [changes] Acceptance gate (unit #6) is mechanically checkable: five `cargo` invocations + two `grep`/`test -f` doc verifications + a regression suite that re-runs the existing `test_layer_boundaries`. No human-judgement gate is left in the loop. The follow-up issue for `Transaction::begin_with_options` is named as a precondition for TD merge, not for landing the parity test itself.

