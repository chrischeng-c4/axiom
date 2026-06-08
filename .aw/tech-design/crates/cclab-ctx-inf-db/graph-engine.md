# Graph Engine

## Overview
<!-- type: overview lang: markdown -->

Graph computation engine with dual execution paths: CPU (rayon threadpool) for small/medium graphs and interactive queries, GPU (wgpu compute shaders) for large-scale analytics. Provides traversal, shortest path, centrality metrics, and community detection.

## Graph Operations Class Hierarchy
<!-- type: dependency lang: mermaid -->

```mermaid
classDiagram
    direction TB

    class GraphOps {
        <<trait>>
        +neighbors(id, direction) Vec~EntityId~
        +degree(id, direction) usize
        +has_edge(src, dst) bool
        +subgraph(ids) SubGraph
    }

    class PathFinder {
        +shortest_path(src, dst, opts) Option~Path~
        +all_paths(src, dst, max_hops) Vec~Path~
        +reachable(src, max_hops) Set~EntityId~
    }

    class CentralityCalc {
        +degree_centrality() Map~EntityId_f64~
        +betweenness_centrality() Map~EntityId_f64~
        +pagerank(damping, iterations) Map~EntityId_f64~
        +eigenvector_centrality() Map~EntityId_f64~
    }

    class CommunityDetector {
        +louvain(resolution) Vec~Community~
        +label_propagation() Vec~Community~
        +connected_components() Vec~Vec~EntityId~~
    }

    class SubGraphExtractor {
        +ego_graph(center, radius) SubGraph
        +k_hop_neighborhood(ids, k) SubGraph
        +induced_subgraph(ids) SubGraph
    }

    class ExecutionBackend {
        <<trait>>
        +execute(op) Result
    }

    class CpuBackend {
        rayon_pool: ThreadPool
    }

    class GpuBackend {
        device: GpuDevice
        pipeline_cache: Map
    }

    PathFinder --> GraphOps : uses
    CentralityCalc --> GraphOps : uses
    CommunityDetector --> GraphOps : uses
    SubGraphExtractor --> GraphOps : uses
    ExecutionBackend <|.. CpuBackend
    ExecutionBackend <|.. GpuBackend
    PathFinder --> ExecutionBackend
    CentralityCalc --> ExecutionBackend
    CommunityDetector --> ExecutionBackend
```

## Execution Routing
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    query[Graph Query] --> estimate[Estimate working set size]
    estimate --> check{Working set > GPU threshold?}
    check -->|yes| gpu_avail{GPU available?}
    check -->|no| cpu[CPU Backend - rayon]
    gpu_avail -->|yes| stage[Stage subgraph to VRAM]
    gpu_avail -->|no| cpu
    stage --> dispatch[Dispatch compute shader]
    dispatch --> readback[Readback results to RAM]
    readback --> merge[Merge with graph state]
    cpu --> merge
    merge --> result[Return results]
```

## BFS/DFS Traversal
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    start[Start BFS from source] --> init[Initialize queue + visited set]
    init --> loop{Queue empty?}
    loop -->|no| dequeue[Dequeue next node]
    dequeue --> filter_temporal{Within temporal window?}
    filter_temporal -->|no| loop
    filter_temporal -->|yes| filter_type{Matches type filter?}
    filter_type -->|no| loop
    filter_type -->|yes| process[Process node + yield to result]
    process --> check_depth{Depth < max_hops?}
    check_depth -->|no| loop
    check_depth -->|yes| expand[Load adjacency page from buffer pool]
    expand --> enqueue[Enqueue unvisited neighbors]
    enqueue --> loop
    loop -->|yes| done[Return traversal result]
```

## PageRank GPU Pipeline
<!-- type: interaction lang: mermaid -->

```mermaid
sequenceDiagram
    participant Q as QueryEngine
    participant G as GraphEngine
    participant B as BufferManager
    participant GPU as GpuDevice

    Q->>G: pagerank(damping=0.85, iter=100)
    G->>B: extract adjacency matrix (CSR format)
    B-->>G: CSR buffers (row_ptr, col_idx, values)
    G->>GPU: create_buffer(row_ptr, col_idx, values, rank_vec)
    G->>GPU: bind_pipeline("pagerank.wgsl")

    loop iterations (100x)
        G->>GPU: dispatch_compute(workgroups)
        GPU-->>GPU: rank[i] = (1-d)/N + d * Σ(rank[j]/out_degree[j])
    end

    G->>GPU: readback(rank_vec)
    GPU-->>G: rank results
    G-->>Q: Map<EntityId, f64>
```

## Path Result Schema
<!-- type: schema lang: json -->

Phase 1 `src/graph.rs::Path` currently ships `{nodes, edges, hop_count, min_confidence}`. The `total_weight` field below is reserved for Phase 3+ weighted-traversal algorithms (Dijkstra / A*); BFS in Phase 1 is unweighted and does not populate it.

```json
{
  "$id": "path-result",
  "title": "Path",
  "type": "object",
  "properties": {
    "nodes": {
      "type": "array",
      "items": { "type": "string", "format": "uuid" },
      "description": "Ordered entity IDs from source to destination"
    },
    "edges": {
      "type": "array",
      "items": { "type": "string", "format": "uuid" },
      "description": "Relation IDs connecting consecutive nodes"
    },
    "total_weight": {
      "type": "number",
      "description": "Sum of edge weights (1/confidence by default). Phase 3+ weighted traversal; absent from Phase 1 BFS output."
    },
    "hop_count": { "type": "integer" },
    "min_confidence": {
      "type": "number",
      "description": "Lowest confidence edge in path — bottleneck indicator"
    }
  }
}
```

## Community Result Schema
<!-- type: schema lang: json -->

```json
{
  "$id": "community-result",
  "title": "Community",
  "type": "object",
  "properties": {
    "community_id": { "type": "integer" },
    "members": {
      "type": "array",
      "items": { "type": "string", "format": "uuid" }
    },
    "size": { "type": "integer" },
    "density": {
      "type": "number",
      "description": "Internal edge density (0..1)"
    },
    "modularity_contribution": {
      "type": "number",
      "description": "This community's contribution to overall modularity score"
    }
  }
}
```
