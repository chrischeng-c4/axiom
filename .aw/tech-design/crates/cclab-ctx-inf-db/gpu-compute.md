# GPU Compute

## Overview
<!-- type: overview lang: markdown -->

GPU acceleration layer via `wgpu` (WebGPU API). Provides cross-platform compute shader execution (Metal on macOS, Vulkan on Linux/Windows). Handles device lifecycle, buffer staging (RAM ↔ VRAM), shader pipeline management, and workgroup dispatch for graph algorithms.

## Device Lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
stateDiagram-v2
    [*] --> Uninitialized
    Uninitialized --> Requesting : request_adapter()
    Requesting --> AdapterReady : adapter found
    Requesting --> Unavailable : no GPU / unsupported
    AdapterReady --> DeviceReady : request_device()
    DeviceReady --> Idle : pipelines compiled
    Idle --> Computing : dispatch()
    Computing --> Idle : readback complete
    Idle --> Lost : device lost event
    Lost --> Requesting : auto-reconnect
    Unavailable --> [*] : fallback to CPU
    DeviceReady --> Lost : device lost event
```

## Buffer Staging Pipeline
<!-- type: interaction lang: mermaid -->

```mermaid
sequenceDiagram
    participant GE as GraphEngine
    participant GM as GpuManager
    participant SB as StagingBuffer (RAM)
    participant DB as DeviceBuffer (VRAM)
    participant CS as ComputeShader

    Note over GE,CS: Upload phase
    GE->>GM: stage_graph(csr_data)
    GM->>SB: write CSR arrays to staging buffer
    GM->>GM: create_command_encoder()
    GM->>DB: copy_buffer_to_buffer(staging → device)
    GM->>GM: submit(encoder)

    Note over GE,CS: Compute phase
    GM->>CS: set_bind_group(device_buffers)
    GM->>CS: dispatch_workgroups(N)
    GM->>GM: submit(compute_encoder)

    Note over GE,CS: Readback phase
    GM->>DB: copy_buffer_to_buffer(device → staging)
    GM->>GM: submit(readback_encoder)
    GM->>SB: map_async(read)
    SB-->>GM: mapped slice
    GM-->>GE: result data
```

## CSR Graph Representation
<!-- type: schema lang: json -->

```json
{
  "$id": "csr-graph",
  "title": "CSRGraph",
  "description": "Compressed Sparse Row format for GPU-friendly adjacency representation",
  "type": "object",
  "properties": {
    "num_nodes": { "type": "integer" },
    "num_edges": { "type": "integer" },
    "row_offsets": {
      "type": "array",
      "items": { "type": "integer", "format": "uint32" },
      "description": "Length = num_nodes + 1. row_offsets[i]..row_offsets[i+1] gives edge range for node i"
    },
    "col_indices": {
      "type": "array",
      "items": { "type": "integer", "format": "uint32" },
      "description": "Length = num_edges. Target node indices"
    },
    "edge_weights": {
      "type": "array",
      "items": { "type": "number", "format": "float32" },
      "description": "Length = num_edges. Edge weights (confidence values)"
    }
  }
}
```

## Compute Shader Catalog
<!-- type: schema lang: json -->

```json
{
  "$id": "shader-catalog",
  "title": "ShaderCatalog",
  "type": "object",
  "additionalProperties": {
    "type": "object",
    "required": ["description", "workgroup_size", "inputs", "outputs"],
    "properties": {
      "description": { "type": "string" },
      "workgroup_size": {
        "type": "array",
        "items": { "type": "integer" },
        "minItems": 3,
        "maxItems": 3
      },
      "inputs": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Named GPU buffer bindings (input)"
      },
      "outputs": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Named GPU buffer bindings (output)"
      },
      "iterations": { "type": "string", "description": "Iteration semantics (if applicable)" }
    }
  },
  "examples": [
    {
      "pagerank": {
        "description": "Iterative PageRank on CSR graph",
        "workgroup_size": [256, 1, 1],
        "inputs": ["row_offsets", "col_indices", "edge_weights", "rank_in"],
        "outputs": ["rank_out"],
        "iterations": "configurable (default 100)"
      },
      "bfs_level": {
        "description": "Level-synchronous BFS (one level per dispatch)",
        "workgroup_size": [256, 1, 1],
        "inputs": ["row_offsets", "col_indices", "frontier", "visited"],
        "outputs": ["next_frontier", "distances"]
      },
      "spmv": {
        "description": "Sparse matrix-vector multiply (building block for eigenvector centrality)",
        "workgroup_size": [256, 1, 1],
        "inputs": ["row_offsets", "col_indices", "values", "vec_in"],
        "outputs": ["vec_out"]
      },
      "community_label_prop": {
        "description": "Label propagation for community detection",
        "workgroup_size": [256, 1, 1],
        "inputs": ["row_offsets", "col_indices", "labels_in"],
        "outputs": ["labels_out", "changed_count"]
      }
    }
  ]
}
```

## GPU Memory Budget
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    request[Graph staging request: N nodes, M edges] --> calc_size["size = N*4 + (N+1)*4 + M*4 + M*4 bytes"]
    calc_size --> check{size <= VRAM budget?}
    check -->|yes| allocate[Allocate device buffers]
    check -->|no| partition[Partition graph into chunks]
    partition --> chunk_loop[Process chunk by chunk]
    chunk_loop --> merge[Merge chunk results on CPU]
    allocate --> dispatch[Dispatch compute]
    dispatch --> result[Return results]
    merge --> result
```
