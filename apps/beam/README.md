# Beam

## Brief

Beam is the GPU vector database in the Axiom service stack.

It owns vector-first storage, GPU ANN indexing, batch ingest, compaction/rebuild,
and vector query execution. It is intentionally separate from `lumen`: Lumen is
a mixed search service across exact, lexical, semantic, perceptual, and
duplicate search; Beam is a GPU-native vector service optimized for vector
indexes and GPU memory tiers.

## Boundaries

- `lumen` owns mixed search and ranking workflows.
- `beam` owns vector-first collections, GPU ANN indexes, and GPU batch
  query/ingest execution.
- `keep` can store large external payloads; Beam stores vectors and vector
  metadata needed for ANN.
- `cube` owns analytical aggregates; Beam owns nearest-neighbor retrieval.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| GPU Vector Index | #769 | partial | verified | conformance | not_ready | GPU flat + IVF-flat + IVF-PQ ANN on wgpu/Metal, recall verified vs oracle; durable segments/memory-tiers/rebuild pending |
| Batch Ingest And Rebuild | #769 | partial | conformance | none | not_ready | batch upsert + CRUD + persistence (save/load); compaction present; offline rebuild pending |
| Vector Query API | #769 | partial | conformance | none | not_ready | kNN + metadata filters over REST (`/v1/collections/{c}/query`); recall gates via ivf_recall |
| HTTP/2 API List | #769 | partial | conformance | none | not_ready | h2c REST live (`beam serve`): health/collections/vectors/query; OpenAPI pending |
| Kubernetes-Native Deployment | #769 | planned | planned | none | not_ready | dedicated StatefulSet/operator shape with GPU scheduling |
| Primary Replicas | #769 | planned | planned | none | not_ready | raft-backed metadata and index lifecycle ownership |
| CLI Interface | #772 | partial | smoke | conformance | not_ready | shell + std verbs (llm/upgrade/issue) landed; vector verbs pending |
| Long-Running Stability | #769 | planned | planned | none | not_ready | GPU index soak, rebuild, failover, and recovery gates |
| Security Hardening | #769 | planned | planned | none | not_ready | collection authz, tenant isolation, audit, and secret rotation |
| Competitor Feature Parity | #769 | partial | conformance | none | not_ready | matrix landed (parity checklist vs Milvus/Qdrant/Faiss/cuVS/pgvector); features tracked F1-F5 |
| Competitor Performance | #769 | partial | dogfood | conformance | not_ready | pinned baseline: beam-GPU beats faiss-CPU batched flat at n>=100k (1.06-2.3x, recall 1.000); loses single-query latency |
| Stateful Service Workload | #769 | planned | planned | none | not_ready | baseline stateful service workload projection |

### Stateful Service Workload

ID: stateful-service-workload
Type: Service
Root WI: #769
Status: confirmed
Surfaces: Durable index, checkpoint state, and stateful replica lifecycle.
EC Dimensions: behavior: pending stateful service workload projection
Required Verification: smoke
Promise:
  Beam projects the shared stateful-service workload baseline.
Gate Inventory:
  - pending: apps/beam/tests/persistence.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| stateful-service-workload-projection | change | #769 | planned | planned | none | pending stateful service workload projection |
| beam-single-node-durable-state | change | #2149 | planned | planned | none | pending atomic serve state recovery |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Root WI: #772
Status: confirmed
Surfaces: CLI: `beam llm`, `beam upgrade`, `beam issue`, vector ingest/query/index, and admin/debug verbs.
EC Dimensions: behavior: pending CLI convention gate - required standard verbs, vector workflow ergonomics, and offline agent docs
Required Verification: smoke, conformance
Promise:
Beam ships an agent-drivable CLI for vector collection, ingest, query, index,
and admin workflows while following the repository-wide CLI convention.
Gate Inventory:
- passing: apps/beam/tests/cli_contract.rs (R1–R6, `cargo test -p beam` green)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| beam-cli-shell | change | #772 | implemented | verified | conformance | apps/beam/tests/cli_contract.rs (R1–R6 green) |
| beam-cli-convention-and-vector-verbs | change | #772 | implemented | verified | conformance | apps/beam/tests/cli_contract.rs (R1–R6 green) |
| beam-vector-verbs | epic | #769 | planned | planned | none | pending vector ingest/query/index verbs |
| beam-canonical-artifact-paths | change | #2147 | planned | planned | none | pending canonical paths |

### Long-Running Stability

ID: long-running-stability
Type: Runtime
Root WI: #769
Status: confirmed
Surfaces: Runtime: GPU index loader, batch ingest, compaction/rebuild worker, query executor, snapshot, and recovery paths.
EC Dimensions: stability: pending long-running vector gate - soak, restart, GPU memory recovery, rebuild safety, bounded memory, and backpressure
Required Verification: conformance, dogfood
Promise:
Beam remains stable under sustained vector ingest, query, index rebuild, and GPU
memory pressure without serving corrupted index state.
Gate Inventory:
- pending: apps/beam/tests/long_running_stability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gpu-vector-soak-and-recovery | epic | #769 | planned | planned | none | pending long-running vector gate |
| beam-test-gate-execution-policy | change | #2146 | planned | planned | none | pending honest test gate execution |

### Security Hardening

ID: security-hardening
Type: Devops
Root WI: #769
Status: confirmed
Surfaces: HTTP/K8s: collection/query authn/authz, tenant/collection isolation, network policy, audit events, secret rotation, and request limits.
EC Dimensions: behavior: pending security gate - auth failure cases, collection isolation, audit emission, secret rotation, and abuse limits
Required Verification: negative, conformance
Promise:
Beam protects vector collections and GPU query APIs with explicit authorization,
auditability, network policy, and managed secret rotation.
Gate Inventory:
- pending: apps/beam/tests/security_hardening.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gpu-vector-security-boundary | epic | #769 | planned | planned | none | pending security hardening gate |
| beam-service-auth | change | #2150 | planned | planned | none | pending service-auth enforcement |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Root WI: #769
Status: confirmed
Surfaces: Docs/Test: GPU/vector database feature matrix against Milvus, Qdrant, Faiss-style, and GPU ANN services.
EC Dimensions: behavior: pending competitor feature gate - collection lifecycle, vector ingest, ANN index build/load, filters, recall diagnostics, rebuild, and GPU scheduling
Required Verification: conformance
Promise:
Beam keeps an explicit GPU/vector feature matrix against established vector
systems, with comparison scope changed only when product requirements change.
Gate Inventory:
- landed: apps/beam/benchmark/competitor-feature-matrix.md (parity checklist + gaps)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gpu-vector-competitor-feature-matrix | epic | #769 | planned | planned | none | pending competitor feature gate |

### Competitor Performance

ID: competitor-performance
Type: RuntimeTool
Root WI: #769
Status: confirmed
Surfaces: Meter/Vat: vector ingest throughput, index build time, query p50/p95, recall, GPU memory pressure, and rebuild cost.
EC Dimensions: efficiency: pending competitor performance gate - pinned external baseline and Beam-owned vector measurements
Required Verification: dogfood
Promise:
Beam maintains a pinned competitor performance baseline and reruns external
benchmarks only when the comparison scope changes or a human explicitly asks.
Gate Inventory:
- landed: apps/beam/benchmark/competitor-performance-baseline.md (beam-GPU beats faiss-CPU batched flat at n>=100k; independently re-verified)
- pending: apps/beam/meter-beam-query.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gpu-vector-competitor-performance-baseline | epic | #769 | planned | planned | none | pending competitor performance gate |
| beam-production-overlap-pipeline | change | #2153 | planned | planned | none | pending high-throughput query serving pipeline |
| four-pillar-throughput-saturation-ddd | change | #769 | planned | planned | none | pending throughput architecture |
| competitive-throughput-ddd-zero-cost | change | #769 | planned | planned | none | pending throughput architecture |
| competitive-throughput-gate-saturate-four-pillars | change | #769 | planned | planned | none | pending throughput architecture |
| competitive-throughput-out-of-core-scaling | change | #769 | planned | planned | none | pending throughput architecture |
| competitive-throughput-gpu-scaling | change | #769 | planned | planned | none | pending throughput architecture |

### GPU Vector Index

ID: gpu-vector-index
Type: RuntimeTool
Root WI: #769
Status: confirmed
Surfaces: GPU engine: vector collection shards, ANN index build/load, GPU memory tier, and host spill policy.
EC Dimensions: behavior: pending vector index conformance gate - build/load/search correctness; efficiency: pending GPU meter gate - throughput, memory, and latency floors
Required Verification: smoke, conformance
Promise:
Beam manages GPU-native vector indexes with explicit memory-tier and rebuild
semantics rather than treating vector search as a Lumen side path.
Gate Inventory:
- passing: apps/beam/tests/gpu_matches_cpu.rs (GPU flat top-k == CPU oracle, L2/Dot/Cosine)
- passing: apps/beam/tests/ivf_recall.rs (IVF-flat exact at full probe; recall grows with nprobe; GPU ADC == CPU ADC; scaling)
- pending: apps/beam/tests/gpu_vector_index.rs (durable index build/load/rebuild lifecycle)
- pending: apps/beam/meter-beam-gpu.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gpu-flat-knn-wgpu-metal | change | #769 | implemented | verified | conformance | apps/beam/tests/gpu_matches_cpu.rs (Apple M1 Max/Metal, recall 1.000) |
| gpu-ivf-pq-ann | change | #769 | implemented | verified | conformance | apps/beam/tests/ivf_recall.rs (IVF-flat 2.7× faster lossless; IVF-PQ ADC exact, recall tunable) |
| gpu-ann-index-lifecycle | epic | #769 | planned | planned | none | pending durable segments, memory tiers, rebuild, and GPU meter gates |
| beam-ddd-wgpu-distance-adapter | change | #2148 | planned | planned | none | pending real wgpu distance adapter |

### Batch Ingest And Rebuild

ID: batch-ingest-and-rebuild
Type: RuntimeTool
Root WI: #769
Status: confirmed
Surfaces: HTTP/Admin: batch ingest, segment compaction, background rebuild, and index promotion controls.
EC Dimensions: behavior: pending ingest/rebuild gate - idempotent batch ingest, compaction, rebuild, and promotion
Required Verification: smoke, conformance
Promise:
Beam ingests vectors in batches and rebuilds/promotes ANN indexes without
serving partially corrupted index state.
Gate Inventory:
- pending: apps/beam/tests/batch_ingest_rebuild.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| batch-ingest-compaction-rebuild | epic | #769 | planned | planned | none | pending ingest/rebuild conformance gate |
| beam-async-cold-vector-repository | change | #2151 | planned | planned | none | pending honest NVMe repository |

### Vector Query API

ID: vector-query-api
Type: RuntimeTool
Root WI: #769
Status: confirmed
Surfaces: HTTP: `/v1/collections/{collection}/query` - nearest-neighbor search, metadata filters, top-k, and recall diagnostics.
EC Dimensions: behavior: pending vector query gate - top-k, filters, recall fixtures, and deterministic pagination; efficiency: pending GPU query gate - p50/p95/top-k throughput
Required Verification: smoke, conformance
Promise:
Beam serves vector nearest-neighbor queries with explicit recall and latency
gates, keeping lexical/perceptual/duplicate search outside its scope.
Gate Inventory:
- pending: apps/beam/tests/vector_query.rs
- pending: apps/beam/meter-beam-query.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| vector-query-recall-latency-contract | epic | #769 | planned | planned | none | pending vector query and GPU query gates |
| beam-ddd-search-correctness | change | #2145 | planned | planned | none | pending ddd search correctness |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Root WI: #769
Status: confirmed
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`, vector collection/index/query routes.
EC Dimensions: behavior: pending h2c/OpenAPI route-list gate - probes, metrics, OpenAPI, and route inventory
Required Verification: smoke, conformance
Promise:
Beam exposes a compact h2c/OpenAPI API list for vector collection, index,
ingest, query, and operator workflows.
Gate Inventory:
- pending: apps/beam/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-route-list | epic | #769 | planned | planned | none | pending h2c/OpenAPI route-list gate |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Root WI: #769
Status: confirmed
Surfaces: K8s: dedicated StatefulSet/operator topology for GPU nodes, storage, probes, backups, PDBs, and scheduling constraints.
EC Dimensions: behavior: pending kustomize/operator render gate - CRD, operator, and GPU instance render; stability: pending kind/GPU deployment dogfood
Required Verification: smoke, dogfood
Promise:
Beam runs as a dedicated k8s-native GPU vector service with operator-managed
GPU scheduling, storage, backup policy, and index lifecycle.
Gate Inventory:
- pending: apps/beam/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dedicated-gpu-vector-service-topology | epic | #769 | planned | planned | none | pending k8s render/dogfood gates |
| beam-k8s-operator-reconcile | change | #2152 | planned | planned | none | pending k8s operator reconcile controller |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Root WI: #769
Status: confirmed
Surfaces: Raft: collection metadata, shard ownership, and index lifecycle state over `libs/raft-core` and `libs/raft-runtime`.
EC Dimensions: stability: pending raft vector failover gate - metadata and index lifecycle state survive failover
Required Verification: conformance, dogfood
Promise:
Beam replicates collection metadata and index lifecycle ownership through raft
while GPU index bytes remain service-owned data-plane state.
Gate Inventory:
- pending: apps/beam/tests/raft_metadata.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-vector-metadata | epic | #769 | planned | planned | none | pending raft metadata failover gate |
