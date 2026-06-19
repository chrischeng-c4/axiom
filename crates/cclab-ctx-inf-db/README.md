# Cclab Ctx Inf Db

## Brief

Cclab Ctx Inf Db is the Rust temporal knowledge graph database crate for
context-inference workflows.

The live crate currently owns the in-memory entity/relation engine, graph and
timeline query helpers, write-behind WAL persistence, snapshots, crash
recovery, corruption-boundary handling, and an ingest benchmark baseline.
Future query-builder, GPU, and inference phases exist in design notes, but this
README only claims the implemented Rust API surfaces and their current gates.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Temporal Knowledge Graph Engine | - | implemented | passing | conformance | not_ready | Rust API for in-memory temporal entity/relation graph operations; ingest baseline is documented but not rerun by default |
| Graph And Timeline Query API | - | implemented | passing | conformance | not_ready | traversal, shortest path, all paths, centrality, connected components, active range, relation-at, and timeline queries |
| Write-Behind Persistence And Recovery | - | implemented | verified | conformance | not_ready | WAL, snapshots, recovery, corruption handling, persistence stats, and concurrent stress coverage |

### Temporal Knowledge Graph Engine

ID: temporal-knowledge-graph-engine
Type: RuntimeTool
Surfaces: Rust API: `cclab_ctx_inf_db::{CtxInfEngine, Entity, Relation, EntityType, RelationType}` - in-memory temporal graph engine and entity/relation model
EC Dimensions: behavior: `cargo test -p cclab-ctx-inf-db` - CRUD, CAS update, type/name filters, relation validation, current/as-of bitemporal state, and engine stats; efficiency: `cargo bench -p cclab-ctx-inf-db --bench ingest_throughput` - ingest throughput baseline documented in BENCHMARKS.md
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Ctx Inf Db provides an in-memory temporal knowledge graph engine for entity/relation CRUD, type indexing, optimistic updates, bitemporal history, and ingest-oriented execution.
Gate Inventory: `cargo test -p cclab-ctx-inf-db`; crates/cclab-ctx-inf-db/tests/engine_test.rs; crates/cclab-ctx-inf-db/tests/bitemporal_test.rs; crates/cclab-ctx-inf-db/BENCHMARKS.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Engine and model behavior contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-ctx-inf-db`; crates/cclab-ctx-inf-db/tests/engine_test.rs; crates/cclab-ctx-inf-db/tests/bitemporal_test.rs; crates/cclab-ctx-inf-db/BENCHMARKS.md |
| Ingest throughput baseline | epic | - | implemented | passing | conformance | crates/cclab-ctx-inf-db/BENCHMARKS.md |

### Graph And Timeline Query API

ID: graph-and-timeline-query-api
Type: DeveloperTool
Surfaces: Rust API: `CtxInfEngine::{reachable, shortest_path, all_paths, degree_centrality, connected_components, active_during, relations_at, timeline}` - graph and temporal query operations
EC Dimensions: behavior: `cargo test -p cclab-ctx-inf-db` - traversal direction, shortest path pruning, all simple paths, connected components, centrality, active range, relation-at, and timeline ordering
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Ctx Inf Db exposes graph traversal, path, centrality, connected-component, temporal range, relation-at, and timeline query APIs over the temporal graph engine.
Gate Inventory: `cargo test -p cclab-ctx-inf-db`; crates/cclab-ctx-inf-db/tests/graph_test.rs; crates/cclab-ctx-inf-db/tests/engine_test.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Graph and timeline query contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-ctx-inf-db`; crates/cclab-ctx-inf-db/tests/graph_test.rs; crates/cclab-ctx-inf-db/tests/engine_test.rs |

### Write-Behind Persistence And Recovery

ID: write-behind-persistence-and-recovery
Type: RuntimeTool
Surfaces: Rust API: `cclab_ctx_inf_db::{PersistenceConfig, PersistenceHandle, RecoveryManager, RecoveryStats, GraphOp}` - WAL, snapshot, recovery, and persistence stats surface
EC Dimensions: behavior: `cargo test -p cclab-ctx-inf-db` - WAL roundtrip, snapshot recovery, WAL delta replay, crash recovery, torn-page/corruption handling, snapshot GC, rotation coherence, and persistence stats; stability: `cargo test -p cclab-ctx-inf-db --test persistence_stress_test` - concurrent ingest, channel-full, durable reopen, and concurrent log/flush stress coverage
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Ctx Inf Db persists graph mutations through a write-behind WAL, creates atomic snapshots, recovers prefix-consistent state after crashes or corruption boundaries, and exposes recovery/persistence stats for operators.
Gate Inventory: `cargo test -p cclab-ctx-inf-db`; crates/cclab-ctx-inf-db/tests/persistence_test.rs; crates/cclab-ctx-inf-db/tests/crash_recovery_test.rs; crates/cclab-ctx-inf-db/tests/torn_page_test.rs; crates/cclab-ctx-inf-db/tests/wal_rotation_test.rs; crates/cclab-ctx-inf-db/tests/persistence_stress_test.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Persistence and recovery behavior contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-ctx-inf-db`; crates/cclab-ctx-inf-db/tests/persistence_test.rs; crates/cclab-ctx-inf-db/tests/crash_recovery_test.rs; crates/cclab-ctx-inf-db/tests/torn_page_test.rs; crates/cclab-ctx-inf-db/tests/wal_rotation_test.rs; crates/cclab-ctx-inf-db/tests/persistence_stress_test.rs |
| Persistence stress risk tracking | epic | - | implemented | verified | negative | `cargo test -p cclab-ctx-inf-db --test persistence_stress_test` |
