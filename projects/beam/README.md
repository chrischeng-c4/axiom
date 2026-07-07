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
| GPU Vector Index | #769 | planned | planned | none | not_ready | GPU ANN index lifecycle and memory tiers |
| Batch Ingest And Rebuild | #769 | planned | planned | none | not_ready | vector ingest, compaction, and offline rebuild |
| Vector Query API | #769 | planned | planned | none | not_ready | nearest-neighbor search with filters and recall gates |
| HTTP/2 API List | #769 | planned | planned | none | not_ready | h2c/OpenAPI endpoint inventory |
| Kubernetes-Native Deployment | #769 | planned | planned | none | not_ready | dedicated StatefulSet/operator shape with GPU scheduling |
| Primary Replicas | #769 | planned | planned | none | not_ready | raft-backed metadata and index lifecycle ownership |
| CLI Interface | #772 | planned | planned | none | not_ready | `beam` CLI for vector ingest/query/admin and agent docs |
| Long-Running Stability | #769 | planned | planned | none | not_ready | GPU index soak, rebuild, failover, and recovery gates |
| Security Hardening | #769 | planned | planned | none | not_ready | collection authz, tenant isolation, audit, and secret rotation |
| Competitor Feature Parity | #769 | planned | planned | none | not_ready | Milvus/Qdrant/Faiss-style GPU vector feature matrix |
| Competitor Performance | #769 | planned | planned | none | not_ready | pinned vector recall/latency baseline, rerun only on scope change |

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
- pending: projects/beam/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| beam-cli-convention-and-vector-verbs | change | #772 | planned | planned | none | pending CLI convention gate |

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
- pending: projects/beam/tests/long_running_stability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gpu-vector-soak-and-recovery | epic | #769 | planned | planned | none | pending long-running vector gate |

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
- pending: projects/beam/tests/security_hardening.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gpu-vector-security-boundary | epic | #769 | planned | planned | none | pending security hardening gate |

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
- pending: projects/beam/benchmark/competitor-feature-matrix.md

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
- pending: projects/beam/benchmark/competitor-performance-baseline.md
- pending: projects/beam/meter-beam-query.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gpu-vector-competitor-performance-baseline | epic | #769 | planned | planned | none | pending competitor performance gate |

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
- pending: projects/beam/tests/gpu_vector_index.rs
- pending: projects/beam/meter-beam-gpu.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gpu-ann-index-lifecycle | epic | #769 | planned | planned | none | pending vector index and GPU meter gates |

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
- pending: projects/beam/tests/batch_ingest_rebuild.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| batch-ingest-compaction-rebuild | epic | #769 | planned | planned | none | pending ingest/rebuild conformance gate |

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
- pending: projects/beam/tests/vector_query.rs
- pending: projects/beam/meter-beam-query.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| vector-query-recall-latency-contract | epic | #769 | planned | planned | none | pending vector query and GPU query gates |

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
- pending: projects/beam/tests/http_api.rs

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
- pending: projects/beam/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dedicated-gpu-vector-service-topology | epic | #769 | planned | planned | none | pending k8s render/dogfood gates |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Root WI: #769
Status: confirmed
Surfaces: Raft: collection metadata, shard ownership, and index lifecycle state over `libs/raft-core` and `libs/raft-host`.
EC Dimensions: stability: pending raft vector failover gate - metadata and index lifecycle state survive failover
Required Verification: conformance, dogfood
Promise:
Beam replicates collection metadata and index lifecycle ownership through raft
while GPU index bytes remain service-owned data-plane state.
Gate Inventory:
- pending: projects/beam/tests/raft_metadata.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-vector-metadata | epic | #769 | planned | planned | none | pending raft metadata failover gate |
