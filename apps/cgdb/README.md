# Cgdb

## Brief

Cgdb is a local graph database for agentic codebase understanding.

It owns the `cgdb` CLI, the `cgdb-daemon` local RPC server, a project catalog,
and a JSONL code/spec graph built from registered project sources and tech
design files. The current implementation is an early local toolchain: it can
model catalog and graph records, expose daemon RPC methods through CLI verbs,
and run deterministic graph smoke tests, but the configured workspace gate is
blocked because `apps/cgdb` is not a root workspace member.

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
Gate Inventory: `cargo test -p cgdb-smoke`; `cargo test --manifest-path apps/cgdb/Cargo.toml`; apps/cgdb/tests/smoke.rs; apps/cgdb/crates/cgdb-cli/src/main.rs; apps/cgdb/crates/cgdb-daemon/src/handlers.rs; apps/cgdb/crates/cgdb-daemon/src/indexer.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| CLI daemon lifecycle and project catalog contract | epic | - | partial | failing | smoke | `cargo test -p cgdb-smoke`; apps/cgdb/crates/cgdb-cli/src/main.rs; apps/cgdb/crates/cgdb-daemon/src/handlers.rs |
| Source and tech-design graph sync contract | epic | - | partial | failing | smoke | `cargo test -p cgdb-smoke`; apps/cgdb/crates/cgdb-daemon/src/indexer.rs; apps/cgdb/crates/cgdb-core/src/graph.rs |

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
Gate Inventory: `cargo test -p cgdb-smoke`; apps/cgdb/tests/smoke.rs; apps/cgdb/crates/cgdb-daemon/src/query.rs; apps/cgdb/crates/cgdb-daemon/src/lens_service.rs; apps/cgdb/crates/cgdb-core/src/lens.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Coverage and impact query contract | epic | - | partial | failing | smoke | `cargo test -p cgdb-smoke`; apps/cgdb/crates/cgdb-daemon/src/query.rs |
| Lens navigation and bounded graph view contract | epic | - | partial | failing | smoke | `cargo test -p cgdb-smoke`; apps/cgdb/crates/cgdb-daemon/src/lens_service.rs; apps/cgdb/crates/cgdb-core/src/lens.rs |

## Design notes (from the v0 kickoff, 2026-06)

Durable decisions folded from the retired session handoff; operational
content (worktree setup, score-era commands) is dropped as superseded.

Positioning — not a general-purpose graph DB. The workload is code / spec /
conversation as a semantic graph, with three real constraints: incremental
indexing (file change → ms-level partial reindex), hybrid retrieval (graph
traversal + vector similarity + keyword composable in one query), and an
agent-friendly API that returns structured context, not raw rows.

Committed design decisions:

1. Typed property graph schema (not RDF, not freeform) — agents need
   predictable structure for prompting.
2. One HNSW vector index per node type (not one global) — agent queries
   usually know their type scope ("similar function" vs "similar spec").
3. GPU (Metal) only for batch vector search (>1000 queries), graph
   embedding, and bulk reindex — kernel-launch overhead beats small
   traversals; normal queries stay CPU + SIMD.

Planned stack (post-v0 layers): single-file mmap + WAL storage, hand-written
CSR adjacency, HNSW vector layer, Metal Performance Shaders via metal-rs,
rayon + NEON SIMD, PyO3/napi-rs bindings later.

Differentiator — temporal graph: every node/edge carries
`commit_hash + timestamp`, enabling queries like "who called this fn 3 days
ago" / "which commit introduced this dependency". JSONL lines carry a
version tag (`{"v":1,...}`) so temporal columns can arrive as a v2 line
variant without breaking replay.
