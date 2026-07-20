<!-- HANDWRITE-BEGIN gap="missing-generator:logic:6c90e9ae" tracker="pending-tracker" reason="scaffold for apps/beam/benchmark/competitor-feature-matrix.md — fill in by hand and update tracker when codegen is ready" -->
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
| **HNSW** | ✅ (hnsw_rs) | ✅ | ✅ | ✅ | ✅ | ✅ | done (F3) |
| CAGRA / GPU graph | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | later |
| **Metadata filtered search** | ✅ (GPU+CPU) | 🟡 | 🟡 | ✅ | ✅ | ✅ | done (F1) |
| **CRUD (delete/update/upsert)** | ✅ (tombstones) | 🟡 | 🟡 | ✅ | ✅ | ✅ | done (F2) |
| **Persistence / durable** | ✅ (serde/bincode) | ✅ (file) | ✅ | ✅ | ✅ | ✅ | done (F4) |
| Collection mgmt (create/drop/list) | ✅ (`beam serve`) | n/a | n/a | ✅ | ✅ | ✅ (DDL) | done (service) |
| HTTP/gRPC query API | ✅ (h2c REST) | ❌ (lib) | ❌ (lib) | ✅ | ✅ | ✅ (SQL) | done (service) |
| OPQ (rotation) | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | P/OPQ |
| Scalar quantization | 🟡 (flat SQ) | ✅ | ✅ | ✅ | ✅ | ❌ | — |
| Distributed / sharding | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | later |
| **Apple Silicon / Metal GPU** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | **beam-only** |
| Portable GPU (Metal+Vulkan) | ✅ (wgpu) | ❌ | ❌ | ❌ | ❌ | ❌ | **beam-only** |

## Performance baseline framing (goal 2)

Beating cuVS/CAGRA on NVIDIA is out of near-term scope (and needs NVIDIA hardware).
The **honest, measurable** performance wins for beam:

1. **BATCHED THROUGHPUT: beam WINS at scale (measured + re-verified, see
   `competitor-performance-baseline.md`).** After a GEMM-tiled + vec4 batched flat
   kernel, **beam-GPU beats faiss-CPU batched at n ≥ 100k** — **1.06× @100k, ~2×
   @1M, ~2.3× @1M–4M**, exact flat, both recall 1.000, same M1 Max. This is a real
   goal-2 win on the throughput axis. (The journey: serial was 9× slower →
   GPU-side top-k → tiled DB-reuse + vec4 crossed over. Tiling + vec4 *together*
   beat AMX; neither alone did.)
2. **SINGLE-QUERY LATENCY: beam does NOT win** — Apple's AMX/Accelerate BLAS is too
   strong for one-query-at-a-time, and beam has a ~1.7 ms per-query GPU dispatch
   floor. beam also loses batched at tiny n=10k (dispatch-bound). The earlier
   blanket "faster on this machine" claim was **falsified then narrowed**: beam wins
   *batched throughput at scale*, not single-query latency.
3. **Memory at scale (real win):** IVF-PQ codes are ~`dim·4/m`× smaller than full
   vectors — **32× smaller at 1M/dim128/m16** (measured) — so beam indexes corpora
   that don't fit as full vectors.
4. **Portability (real win):** beam uses the **Metal GPU** at all; faiss/every
   surveyed competitor has **no GPU path on this Mac**. That's an availability/
   portability win, NOT a raw-speed win.

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

<!-- HANDWRITE-END -->
