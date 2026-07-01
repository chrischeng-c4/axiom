# Beam Competitor Feature Matrix

Defines what **feature parity** means for beam (the GPU vector database) and
tracks beam's current coverage against established vector systems. This is the
`competitor-feature-parity` capability's evidence artifact.

**Policy** (per `../README.md`): the competitor set and feature list are pinned.
Do **not** re-survey competitors or re-run baselines unless the comparison scope
changes or a human explicitly asks. Knowledge cutoff for competitor columns:
2026-01 — the GPU-ANN landscape (esp. cuVS/CAGRA, Milvus-GPU) moves fast; treat
competitor cells as "last surveyed", not live.

## Competitor set

| System | Kind | GPU story | Language |
|---|---|---|---|
| **Faiss** (Meta) | ANN library | GPU = **CUDA only** (IVF, IVF-PQ, some) | C++/Python |
| **cuVS / RAFT** (NVIDIA) | ANN library | **CUDA only**; CAGRA = SOTA GPU graph index | C++/CUDA |
| **Milvus** (Zilliz) | vector DB | GPU via cuVS/Faiss = **CUDA only** | Go/C++ |
| **Qdrant** | vector DB | **CPU only** (HNSW) | **Rust** |
| **Weaviate** | vector DB | CPU (HNSW) | Go |
| **pgvector** | Postgres ext | CPU (IVFFlat, HNSW) | C |
| **LanceDB** | embedded DB | CPU (IVF-PQ), disk-first | Rust |
| **Pinecone** | managed cloud | proprietary | — |

**Structural fact:** every serious *GPU* vector system today is **CUDA/NVIDIA-only**.
None target Apple Silicon / Metal. That gap is beam's wedge (see Positioning).

## Feature parity checklist (rows = "parity" definition; ✅ have · 🟡 partial · ❌ gap)

| Feature | beam (now) | Faiss | cuVS | Milvus | Qdrant | pgvector | → beam slice |
|---|---|---|---|---|---|---|---|
| Metric L2 / cosine / dot | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | done |
| Flat / brute-force kNN | ✅ (GPU) | ✅ | ✅ | ✅ | ✅ | ✅ | done |
| IVF-flat | ✅ (GPU) | ✅ | ✅ | ✅ | ❌ | ✅ | done |
| IVF-PQ (ADC) | ✅ (GPU) | ✅ | ✅ | ✅ | ❌ | ❌ | done |
| **HNSW** | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | **F3** |
| CAGRA / GPU graph | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | later |
| **Metadata filtered search** | ✅ (GPU+CPU) | 🟡 | 🟡 | ✅ | ✅ | ✅ | done (F1) |
| **CRUD (delete/update/upsert)** | ✅ (tombstones) | 🟡 | 🟡 | ✅ | ✅ | ✅ | done (F2) |
| **Persistence / durable** | ❌ (in-mem) | ✅ (file) | ✅ | ✅ | ✅ | ✅ | **F4** |
| Collection mgmt (create/drop/list) | ❌ (CLI stubs) | n/a | n/a | ✅ | ✅ | ✅ (DDL) | **F5** |
| HTTP/gRPC query API | ❌ | ❌ (lib) | ❌ (lib) | ✅ | ✅ | ✅ (SQL) | service slice |
| OPQ (rotation) | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | P/OPQ |
| Scalar quantization | 🟡 (flat SQ) | ✅ | ✅ | ✅ | ✅ | ❌ | — |
| Distributed / sharding | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | later |
| **Apple Silicon / Metal GPU** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | **beam-only** |
| Portable GPU (Metal+Vulkan) | ✅ (wgpu) | ❌ | ❌ | ❌ | ❌ | ❌ | **beam-only** |

## Performance baseline framing (goal 2)

Beating cuVS/CAGRA on NVIDIA is out of near-term scope (and needs NVIDIA hardware).
The **honest, measurable** performance wins for beam:

1. **Availability win (Mac):** on Apple Silicon, every competitor falls back to
   **CPU** (no CUDA). beam runs the query on the **GPU (Metal)**. Head-to-head on
   the *same Mac*: beam-GPU vs Faiss-CPU / pgvector-CPU / Qdrant-CPU. This is a
   real, defensible "faster on this machine" claim.
2. **Memory at scale:** IVF-PQ codes are ~`dim·4/m`× smaller than full vectors
   (measured: **32× smaller at 1M / dim128 / m16**), so beam indexes corpora that
   don't fit as full vectors — verified in `beam bench`.
3. **Not yet a win:** IVF-PQ *query latency* (CPU ADC-table build dominates) and
   recall at scale. Levers tracked as P1 (GPU table-build) + OPQ.

Pinned baseline artifact: `competitor-performance-baseline.md` (to be captured
once F1–F5 land and a stable query surface exists).

## Positioning (honest)

- **Don't** try to out-ANN cuVS/CAGRA — they are years ahead (GPU-resident graph
  indexes, billion-scale, distributed).
- **Do** win on: (a) **GPU vector search where CUDA can't run** — Apple Silicon /
  Metal, portable via wgpu; (b) integration with the axiom agentic stack (vat's
  Metal GPU, clean agent-drivable CLI/service); (c) a lean, correct, `vector-first`
  DB rather than a bolt-on.
- Parity target for "aligned with competitors" = the ✅/🟡 rows above reaching ✅:
  **HNSW, filtered search, CRUD, persistence, collection mgmt, query API** — the
  table-stakes of being a *database*, not just an index.
