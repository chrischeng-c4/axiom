# Cgdb

## Brief

Cgdb is a local graph database for agentic codebase understanding.

It owns the `cgdb` CLI, the `cgdb-daemon` local RPC server, a project catalog,
and a JSONL code/spec graph built from registered project sources and tech
design files. The current implementation is an early local toolchain: it can
model catalog and graph records, expose daemon RPC methods through CLI verbs,
and run deterministic graph smoke tests, but the configured workspace gate is
blocked because `projects/cgdb` is not a root workspace member.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Local Graph Daemon And Project Index | - | partial | failing | smoke | not_ready | CLI/daemon/core exist; configured cargo package gate does not resolve from the root workspace |
| Code Spec Query And Lens Views | - | partial | failing | smoke | not_ready | direct graph smoke coverage exists; full daemon round-trip query and lens verification still needs a runnable workspace gate |

### Local Graph Daemon And Project Index

ID: local-graph-daemon-and-project-index
Type: AgentFirst
Surfaces: CLI: `cgdb daemon start`, `cgdb daemon stop`, `cgdb daemon restart`, `cgdb daemon status`, `cgdb register`, `cgdb unregister`, `cgdb projects`, `cgdb sync`; Daemon RPC: `daemon.status`, `project.register`, `project.unregister`, `project.list`, `project.sync`; Storage: `~/.cgdb/data/catalog.toml`, per-project `graph.jsonl`
EC Dimensions: behavior: `cargo test -p cgdb-smoke` - configured workspace smoke gate for catalog and graph persistence
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cgdb provides a local daemon-backed CLI that registers cclab projects, persists project catalog metadata, syncs source and tech-design files into a versioned code/spec graph, and reports daemon/project status through JSON output.
Gate Inventory: `cargo test -p cgdb-smoke`; `cargo test --manifest-path projects/cgdb/Cargo.toml`; projects/cgdb/tests/smoke.rs; projects/cgdb/crates/cgdb-cli/src/main.rs; projects/cgdb/crates/cgdb-daemon/src/handlers.rs; projects/cgdb/crates/cgdb-daemon/src/indexer.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| CLI daemon lifecycle and project catalog contract | epic | - | partial | failing | smoke | `cargo test -p cgdb-smoke`; projects/cgdb/crates/cgdb-cli/src/main.rs; projects/cgdb/crates/cgdb-daemon/src/handlers.rs |
| Source and tech-design graph sync contract | epic | - | partial | failing | smoke | `cargo test -p cgdb-smoke`; projects/cgdb/crates/cgdb-daemon/src/indexer.rs; projects/cgdb/crates/cgdb-core/src/graph.rs |

### Code Spec Query And Lens Views

ID: code-spec-query-and-lens-views
Type: AgentFirst
Surfaces: CLI: `cgdb query coverage`, `cgdb query impact`, `cgdb lens overview`, `cgdb lens zoom-in`, `cgdb lens zoom-out`, `cgdb lens focus`, `cgdb lens breadcrumb`; Daemon RPC: `query.coverage`, `query.impact`, `lens.overview`, `lens.zoom_in`, `lens.zoom_out`, `lens.focus`, `lens.breadcrumb`; Output: JSON and Mermaid
EC Dimensions: behavior: `cargo test -p cgdb-smoke` - configured smoke gate for graph records plus query/lens follow-up coverage
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cgdb lets agents query code/spec coverage, inspect impact from spec sections to affected code symbols, and request bounded lens views over graph neighborhoods, with deterministic JSON envelopes and optional Mermaid rendering for lens output.
Gate Inventory: `cargo test -p cgdb-smoke`; projects/cgdb/tests/smoke.rs; projects/cgdb/crates/cgdb-daemon/src/query.rs; projects/cgdb/crates/cgdb-daemon/src/lens_service.rs; projects/cgdb/crates/cgdb-core/src/lens.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Coverage and impact query contract | epic | - | partial | failing | smoke | `cargo test -p cgdb-smoke`; projects/cgdb/crates/cgdb-daemon/src/query.rs |
| Lens navigation and bounded graph view contract | epic | - | partial | failing | smoke | `cargo test -p cgdb-smoke`; projects/cgdb/crates/cgdb-daemon/src/lens_service.rs; projects/cgdb/crates/cgdb-core/src/lens.rs |
