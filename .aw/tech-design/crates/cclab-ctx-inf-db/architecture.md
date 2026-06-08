# Architecture

## System Overview
<!-- type: overview lang: markdown -->

CtxInfDB is a temporal knowledge graph database built for intelligence analysis workloads. It combines four resource tiers (GPU, CPU, RAM, Disk) into a unified engine that supports entity/relation CRUD, graph analytics, timeline construction, and inference.

## System Layers
<!-- type: dependency lang: mermaid -->

```mermaid
classDiagram
    direction TB

    class QueryAPI {
        QueryBuilder
        ResultSet
        Cursor
    }

    class InferenceEngine {
        RuleEngine
        PatternMatcher
        ConfidenceScorer
        AnomalyDetector
    }

    class GraphEngine {
        Traversal
        ShortestPath
        Centrality
        CommunityDetection
    }

    class TemporalEngine {
        IntervalIndex
        TimelineBuilder
        TemporalOverlap
        EventSequencer
    }

    class GPUCompute {
        DeviceManager
        ShaderPipeline
        BufferStaging
        KernelDispatch
    }

    class CPUCompute {
        RayonPool
        ParallelBFS
        RuleEvaluator
    }

    class BufferManager {
        PageCache
        EvictionPolicy
        TierPromotion
        TierDemotion
    }

    class StorageEngine {
        PageStore
        WAL
        SnapshotManager
        MmapRegion
    }

    QueryAPI --> InferenceEngine
    QueryAPI --> GraphEngine
    QueryAPI --> TemporalEngine
    InferenceEngine --> GraphEngine
    InferenceEngine --> TemporalEngine
    GraphEngine --> GPUCompute
    GraphEngine --> CPUCompute
    TemporalEngine --> CPUCompute
    GPUCompute --> BufferManager
    CPUCompute --> BufferManager
    BufferManager --> StorageEngine
```

## Storage Tier Architecture
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TB
    subgraph VRAM["VRAM (GPU Memory)"]
        adj_matrix[Adjacency Matrix Buffers]
        compute_scratch[Compute Scratch Space]
        result_buf[Result Buffers]
    end

    subgraph RAM["RAM (System Memory)"]
        buffer_pool[Buffer Pool / Page Cache]
        indices[Type Index + Temporal Index + Adjacency Index]
        hot_graph[Hot Subgraph Working Set]
    end

    subgraph DISK["Disk (Persistent)"]
        data_pages[Data Pages - Nodes + Edges + Properties]
        wal_files[WAL Files]
        snapshots[Snapshots]
        mmap_region[mmap Region - Cold Pages]
    end

    adj_matrix <-->|stage / readback| buffer_pool
    compute_scratch <-->|stage / readback| buffer_pool
    buffer_pool <-->|page fault / evict| data_pages
    buffer_pool <-->|demand page| mmap_region
    hot_graph --- buffer_pool
    indices --- buffer_pool
    wal_files -.->|recovery| buffer_pool
    snapshots -.->|recovery| buffer_pool
```

## Module Dependency Graph
<!-- type: dependency lang: mermaid -->

Reflects current Phase 1 + Phase 2 module layout (under `crates/cclab-ctx-inf-db/src/`). Aspirational modules (Phase 2.5–5) are tagged explicitly; they are placeholders for future work and do **not** exist in the codebase yet. The WAL (`WalWriter` / `WalReader` / `WalEntry`) lives in the external `cclab-wal` crate, not as a top-level module of this crate. Indices (type-index, adjacency) are inline `DashMap` fields on `CtxInfEngine::new`, not a separate module.

```mermaid
classDiagram
    direction LR

    class lib {
        re-exports
    }
    class types {
        Entity
        Relation
        PropertyValue
        EntityType
        RelationType
    }
    class error {
        CtxInfError
    }
    class engine {
        CtxInfEngine
        create/update/delete_entity
        create/update/delete_relation
        flush / shutdown / create_snapshot
    }
    class graph {
        bfs / dfs / shortest_path
        degree / neighbors
    }
    class temporal {
        active_at / overlap
        TemporalRange semantics
    }
    class storage {
        GraphOp (wal_ops)
        PersistenceHandle (handle)
        RecoveryManager (recovery)
        SnapshotWriter/Loader (snapshot)
        Page format (page)
    }
    class cclab_wal["cclab-wal (external)"] {
        WalWriter
        WalReader
        WalEntry
    }

    class buffer["buffer (aspirational — Phase 2.5)"]
    class query["query (aspirational — Phase 3)"]
    class gpu["gpu (aspirational — Phase 4)"]
    class inference["inference (aspirational — Phase 5)"]

    lib --> engine
    lib --> types
    lib --> error
    engine --> types
    engine --> error
    engine --> graph
    engine --> temporal
    engine --> storage
    storage --> cclab_wal
    storage --> types
    storage --> error
    graph --> types
    temporal --> types
    types --> error

    query -.-> engine
    query -.-> graph
    query -.-> temporal
    inference -.-> graph
    inference -.-> temporal
    graph -.-> gpu
    engine -.-> buffer
    buffer -.-> storage
```
