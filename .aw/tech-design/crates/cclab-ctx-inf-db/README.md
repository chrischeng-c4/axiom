# cclab-ctx-inf-db

Context Inference Database — crate: `cclab-ctx-inf-db`

Temporal knowledge graph database with GPU-accelerated graph analytics and rule-based inference engine. Designed for open-source intelligence analysis: entity tracking, relationship mapping, timeline construction, and pattern-based anomaly detection.

## Specs

| Spec | Format | Scope |
|------|--------|-------|
| [architecture](./architecture.md) | Mermaid (C4, dependency, layer) | System structure, storage tiers, resource mapping |
| [data-model](./data-model.md) | JSON Schema + Mermaid erDiagram | Entity, Relation, PropertyValue, CtxInfError, temporal ranges |
| [storage-engine](./storage-engine.md) | Mermaid (state, flowchart) + JSON Schema | Page-based storage, buffer pool, WAL, transactions, mmap |
| [graph-engine](./graph-engine.md) | Mermaid (flowchart, class) + JSON Schema | Traversal, shortest path, centrality, community detection |
| [temporal-engine](./temporal-engine.md) | Mermaid (state, sequence) + JSON Schema | Timeline queries, interval index, event sequencing |
| [inference-engine](./inference-engine.md) | Mermaid (flowchart, state) + JSON Schema | Rule engine, pattern matching, confidence propagation |
| [gpu-compute](./gpu-compute.md) | Mermaid (sequence, flowchart) | wgpu device mgmt, compute shaders, RAM-VRAM transfer |
| [query-api](./query-api.md) | JSON Schema (OpenRPC) | Query builder, fluent API, result types |
| [competitive-landscape](./competitive-landscape.md) | Markdown (feature matrix) | Market positioning vs. Neo4j/Datomic/Kuzu/cuGraph/etc.; target OSINT use cases |
| [design-decisions](./design-decisions.md) | Markdown (decision log) | D1–D5: bitemporal, Datalog query, confidence math, provenance, GPU tiering |

## Resource Model

| Resource | Role |
|----------|------|
| **GPU / VRAM** | Graph algorithm acceleration (PageRank, BFS, adjacency matrix ops, community detection) |
| **RAM** | Hot subgraph, buffer pool, indices, active query working sets |
| **CPU** | Multi-threaded traversal, inference rule evaluation, query planning, serialization |
| **Disk** | Full graph persistence (page-based), WAL, snapshots, mmap for overflow |

## Phase Plan

| Phase | Deliverable | Deps |
|-------|-------------|------|
| 1 — Foundation | `types`, `error`, `engine` (in-memory CRUD), `graph` (BFS, shortest_path, centrality, components), `temporal` (timeline, `TemporalRange`) | — |
| 2 — Persistence | `storage/{wal_ops, handle, snapshot, recovery}` (via `cclab-wal`) — write-behind durability + recovery | Phase 1 |
| 2.5 — Buffer Pool | `storage/page` as live format (currently snapshot-only), `buffer` (pool manager), `mmap` overflow | Phase 2 |
| 3 — Query | `query/` (builder, fluent API, result types) | Phase 2.5 |
| 4 — GPU | `gpu/` (wgpu device, compute shaders, VRAM staging) | Phase 3 |
| 5 — Inference | `inference/` (rules, patterns, confidence scoring, anomaly detection) | Phase 3+4 |
