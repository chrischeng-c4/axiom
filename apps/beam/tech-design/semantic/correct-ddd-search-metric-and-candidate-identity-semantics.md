---
id: '2145'
summary: "Correct DDD search metric and candidate identity semantics in the high-throughput pipeline."
capability_refs:
  - id: vector-query-api
    role: primary
    claim: beam-ddd-search-correctness
    coverage: full
    rationale: "Ensures the DDD search path preserves the metric and candidate identity semantics promised by Beam's vector query capability."
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: ddd-search-pipeline-correctness
entry: start
nodes:
  start: { kind: start, label: "Start execute_batch" }
  validate_dim: { kind: decision, label: "Validate query dim" }
  resolve_ids: { kind: process, label: "Resolve IDs to offsets 1-to-1" }
  fetch_vectors: { kind: process, label: "Fetch target vectors from repository" }
  validate_bytes: { kind: decision, label: "Validate fetched byte alignment & size" }
  decode_vectors: { kind: process, label: "Decode target vectors & validate count" }
  compute_dist: { kind: process, label: "Compute distance via metric-aware Calculator" }
  zip_sort: { kind: process, label: "Zip resolved candidate IDs, sort deterministically" }
  truncate_k: { kind: process, label: "Truncate to k" }
  success: { kind: terminal, label: "Return success" }
  error_dim: { kind: terminal, label: "Return DimensionMismatch error" }
  error_bytes: { kind: terminal, label: "Return ByteAlignmentMismatch error" }
  error_count: { kind: terminal, label: "Return VectorCountMismatch error" }
edges:
  - { from: start, to: validate_dim }
  - { from: validate_dim, to: resolve_ids, label: "valid" }
  - { from: validate_dim, to: error_dim, label: "invalid" }
  - { from: resolve_ids, to: fetch_vectors }
  - { from: fetch_vectors, to: validate_bytes }
  - { from: validate_bytes, to: decode_vectors, label: "valid" }
  - { from: validate_bytes, to: error_bytes, label: "invalid" }
  - { from: decode_vectors, to: compute_dist }
  - { from: compute_dist, to: zip_sort }
  - { from: zip_sort, to: truncate_k }
  - { from: truncate_k, to: success }
---
flowchart TD
    start([Start execute_batch]) --> validate_dim{Validate query dim?}
    validate_dim -->|Yes| resolve_ids[Resolve IDs to offsets 1-to-1]
    validate_dim -->|No| error_dim[Return DimensionMismatch error]
    resolve_ids --> fetch_vectors[Fetch target vectors from repository]
    fetch_vectors --> validate_bytes{Validate byte alignment & size?}
    validate_bytes -->|Yes| decode_vectors[Decode target vectors & validate count]
    validate_bytes -->|No| error_bytes[Return ByteAlignmentMismatch error]
    decode_vectors --> compute_dist[Compute distance via metric-aware Calculator]
    compute_dist --> zip_sort[Zip resolved candidate IDs, sort deterministically]
    zip_sort --> truncate_k[Truncate to k]
    truncate_k --> success([Return success])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/src/domain/ports.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "pub trait DistanceCalculator"
    description: "Make compute_batched metric-aware by adding a metric parameter to the port interface."
  - path: apps/beam/src/domain/scheduler.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "pub struct PipelineScheduler"
    description: "Validate query dimension, alignment, decoded count, and score count; preserve 1-to-1 mapping of candidate ID and offset."
  - path: apps/beam/src/infrastructure/wgpu_engine.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "impl DistanceCalculator for WgpuDistanceEngine"
    description: "Implement correct L2, Dot, and Cosine metric scoring fallback on the CPU."
  - path: apps/beam/tests/throughput_pipeline.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "struct MockDistanceCalculator"
    description: "Update MockDistanceCalculator to be metric-aware, and add comprehensive negative/regression tests for dimension validation, metric ordering, and missing candidates."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: ddd-search-correctness-verification
requirements:
  boundary_validation:
    id: R3
    text: "Reject malformed query dimensions and truncated/extra vector bytes without panicking or fabricating results, returning typed errors."
    kind: negative
    risk: medium
    verify: cargo test -p beam --test throughput_pipeline test_boundary_validation
  identity_preservation:
    id: R2
    text: "Preserve a one-to-one association between resolved candidate ID, storage offset, decoded vector, and returned score. A missing middle candidate offset cannot shift a later score onto the wrong ID."
    kind: regression
    risk: high
    verify: cargo test -p beam --test throughput_pipeline test_missing_candidate_offset
  metric_aware_port:
    id: R1
    text: "Make the distance-calculation port explicitly metric-aware for L2, Dot, and Cosine."
    kind: functional
    risk: low
    verify: cargo test -p beam --test throughput_pipeline test_metric_aware_port
---
flowchart TD
    r1[R1 metric aware port] --> cargo_test_p_beam_test_throughput_pipeline_test_metric_aware_port[cargo test -p beam --test throughput_pipeline test_metric_aware_port]
    r2[R2 identity preservation] --> cargo_test_p_beam_test_throughput_pipeline_test_missing_candidate_offset[cargo test -p beam --test throughput_pipeline test_missing_candidate_offset]
    r3[R3 boundary validation] --> cargo_test_p_beam_test_throughput_pipeline_test_boundary_validation[cargo test -p beam --test throughput_pipeline test_boundary_validation]
```
