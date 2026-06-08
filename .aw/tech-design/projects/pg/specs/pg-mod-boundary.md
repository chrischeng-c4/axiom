---
id: pg-mod-boundary
fill_sections: [changes]
---

## Changes
<!-- type: changes lang: yaml -->

```yaml
# Mechanical mod-boundary refactor inside projects/pgkit/pg/. Pure file
# moves (git mv) + new mod.rs roots + flat re-export shims at lib.rs +
# one new boundary-audit test. Behaviour does not change; cargo check
# --workspace and the existing test suite remain green.

changes:
  # --- 1. Driver layer (psycopg / asyncpg equivalent) ----------------
  - path: projects/pgkit/pg/src/driver/mod.rs
    action: create
    description: |
      New driver layer root. Declares the sub-modules moved under
      `driver/` and re-exports their public surface so the crate root
      can `pub use driver::*;`:

          pub mod blocking;
          pub mod bulk;
          pub mod connection;
          pub mod dialect;
          pub mod executor;
          pub mod row;
          pub mod transaction;
          pub mod types;

          pub use blocking::{Connection as _, ...};  // see Verification
          pub use connection::*;
          pub use transaction::*;
          pub use row::*;
          pub use types::*;
          pub use bulk::*;
          pub use dialect::*;
          pub use executor::*;

      Exact `pub use` set is whatever `lib.rs` currently re-exports from
      these files. The acceptance gate (`cargo check --workspace` +
      `cargo test`) is the source of truth for "did we miss any
      re-export".

  - path: projects/pgkit/pg/src/connection.rs
    action: move
    to: projects/pgkit/pg/src/driver/connection.rs
    description: |
      `git mv` — no content change. Internal `use crate::...` paths
      that referenced sibling files (now under `driver/`) keep working
      because `crate::connection`-style references are not used; the
      file uses absolute `crate::{Result, Error}` etc. which still
      resolve via lib.rs re-exports.

  - path: projects/pgkit/pg/src/transaction.rs
    action: move
    to: projects/pgkit/pg/src/driver/transaction.rs
    description: git mv — driver layer.

  - path: projects/pgkit/pg/src/executor.rs
    action: move
    to: projects/pgkit/pg/src/driver/executor.rs
    description: git mv — driver layer.

  - path: projects/pgkit/pg/src/row.rs
    action: move
    to: projects/pgkit/pg/src/driver/row.rs
    description: git mv — driver layer.

  - path: projects/pgkit/pg/src/types.rs
    action: move
    to: projects/pgkit/pg/src/driver/types.rs
    description: git mv — driver layer.

  - path: projects/pgkit/pg/src/bulk.rs
    action: move
    to: projects/pgkit/pg/src/driver/bulk.rs
    description: git mv — driver layer.

  - path: projects/pgkit/pg/src/dialect.rs
    action: move
    to: projects/pgkit/pg/src/driver/dialect.rs
    description: git mv — driver layer.

  - path: projects/pgkit/pg/src/blocking
    action: move
    to: projects/pgkit/pg/src/driver/blocking
    description: |
      git mv the whole directory. The blocking facade is a driver
      concern (wraps async Connection/Transaction/MigrationRunner with
      Arc<Runtime>); under `driver/blocking/` its inner imports of
      `crate::{Connection, Transaction}` still resolve via lib.rs
      re-exports.

  # --- 2. ORM layer (SQLAlchemy equivalent) --------------------------
  - path: projects/pgkit/pg/src/orm/mod.rs
    action: create
    description: |
      New ORM layer root. Declares the moved sub-modules and re-exports
      their public surface for crate-root `pub use orm::*;`:

          pub mod auto_detect;
          pub mod backref;
          pub mod compat;
          pub mod query;
          pub mod schema;
          pub mod validation;

          pub use auto_detect::*;
          pub use backref::*;
          pub use compat::*;
          pub use query::*;
          pub use schema::*;
          pub use validation::*;

  - path: projects/pgkit/pg/src/schema.rs
    action: move
    to: projects/pgkit/pg/src/orm/schema.rs
    description: git mv — ORM layer.

  - path: projects/pgkit/pg/src/backref.rs
    action: move
    to: projects/pgkit/pg/src/orm/backref.rs
    description: git mv — ORM layer.

  - path: projects/pgkit/pg/src/validation.rs
    action: move
    to: projects/pgkit/pg/src/orm/validation.rs
    description: git mv — ORM layer.

  - path: projects/pgkit/pg/src/compat.rs
    action: move
    to: projects/pgkit/pg/src/orm/compat.rs
    description: git mv — ORM layer.

  - path: projects/pgkit/pg/src/auto_detect.rs
    action: move
    to: projects/pgkit/pg/src/orm/auto_detect.rs
    description: git mv — ORM layer.

  - path: projects/pgkit/pg/src/query
    action: move
    to: projects/pgkit/pg/src/orm/query
    description: git mv the whole query/ directory — ORM layer.

  # --- 3. Migrate layer (Alembic equivalent) -------------------------
  - path: projects/pgkit/pg/src/migrate/mod.rs
    action: create
    description: |
      New migrate layer root. Declares the moved sub-modules and
      re-exports their public surface:

          pub mod history_vis;
          pub mod migration;

          pub use history_vis::*;
          pub use migration::*;

  - path: projects/pgkit/pg/src/migration
    action: move
    to: projects/pgkit/pg/src/migrate/migration
    description: git mv the whole migration/ subtree — migrate layer.

  - path: projects/pgkit/pg/src/history_vis.rs
    action: move
    to: projects/pgkit/pg/src/migrate/history_vis.rs
    description: git mv — migrate layer.

  # --- 4. Crate root rewires ------------------------------------------
  - path: projects/pgkit/pg/src/lib.rs
    action: modify
    anchor: 'pub mod blocking;'
    description: |
      Replace the flat module declarations with the new layered ones.
      Remove every `pub mod <name>;` that names a file now under
      `driver/`, `orm/`, or `migrate/`, then add the three layer
      declarations plus flat-surface re-exports:

          pub mod driver;
          pub mod orm;
          pub mod migrate;

          // Cross-cutting (unchanged).
          pub mod cli;
          pub mod metrics;

          // Flat compatibility surface (existing call sites resolve
          // at the original cclab_pg::<Foo> path).
          pub use driver::*;
          pub use orm::*;
          pub use migrate::*;

          // Preserve the historic public namespace for the blocking
          // facade: cclab_pg::blocking::Connection ==
          // cclab_pg::driver::blocking::Connection.
          pub use driver::blocking;

      Re-exports in `driver::*` / `orm::*` / `migrate::*` provide every
      item that was previously a direct crate-root export. The
      acceptance gate (cargo check --workspace) catches any miss.

  # --- 5. Boundary audit test -----------------------------------------
  - path: projects/pgkit/pg/tests/test_layer_boundaries.rs
    action: create
    description: |
      New integration test that fails the build if a `driver/` file
      contains `use crate::orm::` or `use crate::migrate::`. Walks
      `src/driver/` recursively, reads each `.rs` file, asserts neither
      forbidden prefix appears. Notes inline that once Phase B
      extracts the layers into separate crates this grep test retires
      and the real Cargo.toml dep graph becomes the enforcement.

  # --- 6. Acceptance gate --------------------------------------------
  - path: VERIFY
    action: verify
    description: |
      All must pass in order (operator-side gate):

        cargo check  -p cclab-pg
        cargo check  --workspace
        cargo test   -p cclab-pg --lib
        cargo test   -p cclab-pg --test test_layer_boundaries

      Plus the existing test suite stays green:

        cargo test   -p cclab-pg

      And nothing under driver/ references higher layers:

        ! grep -rn 'use crate::orm::\|use crate::migrate::' \
            projects/pgkit/pg/src/driver

      Public-surface diff sanity: no item that was reachable as
      `cclab_pg::<Foo>` before the change becomes unreachable after.
      A quick smoke check is to build the dependent crates:

        cargo check  -p cclab-pg-cli -p cclab-pg-mamba

      Both must succeed unchanged (no source edits in those crates).
```

# Reviews

### Review 1
**Verdict:** approved

- [changes] Each step has a concrete `path`, `action` (`create` / `move` / `modify` / `verify`), and a description that is precise enough to execute mechanically. Sequencing is dependency-correct: layer roots come first (`driver/mod.rs`, `orm/mod.rs`, `migrate/mod.rs`), then the `git mv` of each constituent file/directory, then the `lib.rs` rewire that flips from flat `pub mod foo;` to `pub mod driver; pub use driver::*;`. At no intermediate step does the workspace fail to build, because flat-surface re-exports preserve every prior `cclab_pg::<Foo>` path.
- [changes] The acceptance gate at step 6 is mechanically checkable (`cargo check -p cclab-pg`, `cargo check --workspace`, `cargo test -p cclab-pg`, `cargo test -p cclab-pg --test test_layer_boundaries`, plus a negative `! grep -rn 'use crate::orm::\|use crate::migrate::' projects/pgkit/pg/src/driver`). The smoke check on `cclab-pg-cli` / `cclab-pg-mamba` confirms downstream callers are not touched. Boundary-audit grep retirement once Phase B extracts crates is noted in-spec (per Review 1 of the issue).
