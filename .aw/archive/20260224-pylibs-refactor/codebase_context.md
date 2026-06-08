---
change_id: pylibs-refactor
type: codebase_context
created_at: 2026-02-24T09:48:41.616383+00:00
updated_at: 2026-02-24T09:48:41.616383+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - prism_references
  - prism_diagnostics
  - prism_check
---

# Codebase Context

## Analyzed Files

- **crates/cclab-pg/src/pyo3_bindings/** — #81 ALREADY SPLIT: pg bindings decentralized from nucleus, 8 files all <500 lines (largest: query_builder.rs 411 lines)
  - symbols: `connection.rs`, `enums.rs`, `helpers.rs`, `mod.rs`, `query_builder.rs`, `schema.rs`, `transaction.rs`, `window.rs`
- **crates/cclab-queue/src/pyo3_bindings/mod.rs** — #82 STILL NEEDS SPLIT: 924 lines in single file (under 1000 hard limit but over 500 soft limit)
  - symbols: `PyTask`, `PyChain`, `PyGroup`, `PyChord`, `PySignature`
- **crates/cclab-mongo/src/pyo3_bindings/document.rs** — #85 STILL NEEDS SPLIT: 728 lines (over 500 soft limit). Other mongo bindings already split into 10 files
  - symbols: `PyDocument`, `save`, `delete`, `reload`, `find`, `find_one`, `count`, `aggregate`
- **crates/cclab-mongo/src/pyo3_bindings/query.rs** — #85 related: 480 lines (approaching 500 soft limit)
  - symbols: `PyQuery`
- **crates/cclab-titan/tests/** — #135 test coverage target - existing tests, missing pool/constraints/cascade/upsert
- **crates/cclab-titan/src/pool.rs** — #135 connection pool - 0 tests
  - symbols: `Pool`, `PoolConfig`
- **crates/cclab-titan/src/error.rs** — #135 error types - 0 constraint violation tests
  - symbols: `TitanError`
- **crates/cclab-quasar/src/pyo3_bindings/** — #138 API parity target
  - symbols: `PythonHandler`, `PyWebSocket`, `register_module`
- **crates/cclab-shield/src/** — #189 performance target (json->model 1.5-1.8x vs pydantic)
  - symbols: `validate_types_fast`
- **crates/cclab-http/** — #455 OLD crate - still exists alongside cclab-fetch
- **crates/cclab-fetch/** — #455 NEW crate - already exists with client.rs, config.rs, error.rs, lib.rs, middleware.rs, pyo3_bindings/, request.rs, response.rs
- **Cargo.toml** — Workspace root - needs cleanup of cclab-http references

## Prism Results

- **prism_diagnostics** (query: `file line counts`)
  - #81 RESOLVED: pg bindings already split (8 files, max 411 lines). #82 NEEDS WORK: queue mod.rs 924 lines. #85 NEEDS WORK: mongo document.rs 728 lines, query.rs 480 lines
- **prism_references** (query: `cclab-http vs cclab-fetch`)
  - Both crates exist. cclab-http may still be referenced by downstream crates. Need to complete migration and remove old crate
- **prism_check** (query: `cclab-titan tests`)
  - Existing tests compile. Missing test files for pool, constraints, cascade, upsert
