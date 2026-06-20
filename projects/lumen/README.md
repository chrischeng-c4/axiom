# lumen

## Brief

A K8s-native, **log-replicated search specialist**. Five flavors of
"find":

- **Exact** — `keyword` / `number` / `set`
- **Lexical** — `text` (BM25, with tokenize built in)
- **Semantic** — `vector` (CPU: HNSW + exact flat brute-force)
- **Perceptual / structural** — `hash` (pHash / SimHash / b-bit MinHash, Hamming distance)
- **Duplicates** — find which `external_id`s share the same value (a search-flavor of group-by; bounded, posting-list-cheap)

The caller owns the representation:

- Embeddings? **Caller** runs CLIP / BGE / Whisper / VideoMAE; lumen never owns a model artefact.
- Perceptual hashes? **Caller** runs `imagehash` / `datasketch`; lumen indexes the bits.
- Lexical tokenization? **lumen** does it — that's the one place caller doesn't compute (`whitespace_lower` / `ngram` / `jieba`).

The caller also owns the **source of truth**: lumen is a parallel derived index,
never the system of record or an analytics engine — documents are *not* a lumen
concept, only the caller's `external_id` is.

- **Log-driven, derived, rebuildable**. A write is *published to a log*,
  not applied where it lands; every serving node tails the log and folds
  it into its own index. Lossable but rebuildable from the log + the
  caller.
- **Client API on `:7373`** (HTTP/1.1 + HTTP/2 cleartext — REST clients
  need nothing special; see [HTTP](#http--clients)).
- **Sharded**: `crc32(collection_id) % shard_count` routes on the client.
  Shard count is install-time, not online-changeable.
- **Agent-first offline integration surface**: `lumen spec` emits the exact
  machine schema, including `lumen spec --format openapi-yaml` for LLM-readable
  OpenAPI, while `lumen llm outline|workflow|integration|quickstart|recipes`
  lets an agent pick the smallest context needed to wire lumen into an app
  without a docs site or running server.

## Capabilities

**What lumen is for.** lumen is the **search layer for an OLTP system of record**
— Postgres / AlloyDB / MongoDB. It fixes what those stores are weak at — real
BM25, filter-correct vector kNN, hybrid lexical+semantic retrieval, CJK — while
the OLTP store stays the source of truth: lumen holds no documents, only index
bits, and is rebuilt from the caller's data + the log. Output is ranked
`external_id`s for an **agent** (or app) to hydrate against its own store. There
is deliberately **no Kibana, no analytics, no bundled ingestion pipeline** —
ingestion is the **caller's own pub/sub into `POST /index`** (CDC / logical
replication / app writes), and lumen bundles no connector.

The pitch is not only *what it finds* but *how it runs*: **fast** (early-terminating
planner, in-memory serving, no GC pauses), **lightweight** (a single Rust binary,
no JVM; RAM is a working set, scale bounded by cheap disk), **operable without a
DBA** (stateless nodes, no consensus / leader election to run, search load off the
OLTP primary), and **stable** (survives broker- and pod-kill with byte-identical
results) — first-class, gated promises under
[Operational characteristics](#operational-characteristics-operability--speed--footprint--stability).

This is the human-confirmed product promise for lumen, in `aw capability`
Markdown-table form. Each `###` below is a capability root: a contract table
(promise + required verification + gate inventory) followed by a work-root table
(the epics/gaps that deliver it, with honest `impl`/`verification` state). The
prose sections further down are the detailed reference for each capability;
gate-inventory paths point at the real tests/scripts/manifests that prove them.
Statuses are deliberately conservative — `auditing` means "built and gated, not
yet formally `--verify`-proven"; `candidate` means "promised, partially shipped".

The capability roots group into **three pillars**, each aligned across its CLI
surface, this capability map, and its EC contract:

- **agent-first** — the offline integration surface (`lumen spec` / `lumen llm`).
- **serve / search** — the running engine (`lumen serve`): the product core plus its runtime properties.
- **devops-operation** — the shipped deployment defaults (`lumen k8s` + the Dockerfile / kustomize / operator artifacts; lumen ships them, it does not deploy).

**Honest scope (do not over-claim):**

- **Ingestion is the caller's own pub/sub** into `POST /index` (CDC / logical
  replication / app writes). lumen bundles no connector and owns no upstream
  subscription — it is a parallel derived index, rebuildable from the source + log.
- **Rust speed target is broader than today's gate.** Because lumen is a focused
  Rust index (no JVM, no document store, no analytics surface), the product target
  is **several-x faster than OpenSearch across every search shape**. Current release
  evidence now clears the 1M in-memory and segment-disk search gates vs OpenSearch
  on every search cell, and clears Postgres on every non-home-turf or native-binary
  search cell. pg cheap btree predicates remain
  explicit EXEMPT home turf for the public HTTP/JSON serial loopback comparison,
  but the same cheap predicates now have a prepared native binary gate over Unix
  socket/TCP fallback. qps10/qps100/qps1000 are paced, usable, and strict-gated
  through `LUMEN_PERF_STRICT=1`; the latest retained 1M evidence clears every
  qps tier against OpenSearch, including a dedicated qps10 rerun whose lowest
  retained row is `filtered_search` at **2.66x**. qps10 is still close to the
  co-located low-QPS floor, so guarded harness-bound retries and isolated-host
  repeats remain part of making "several-x everywhere" release-stable.
  Write-path QPS is also report-only by default but strict-gated by the same
  perf-strict mode. Isolated load hosts are the next lever for making
  "several-x everywhere" a default CI claim.
- **Wide-range filters** drive from an on-disk **sorted-value range index** (the
  distinct values as a page-aligned ascending column, binary-searched on the mmap)
  + per-value posting lists. The standalone disk `range` planner streams only the
  selected sorted-value window and the keyword-filtered sort path drives from lazy
  `(keyword term -> docs sorted by number)` skip lists, so both now clear the
  OpenSearch disk target. pg's cheap HTTP range predicate remains exempt home turf;
  the prepared native binary path gates the same range predicate against pg's
  prepared Unix-socket path.
- **Vector search is CPU-only** in this version (HNSW + exact flat brute-force).
  The **flat-cpu** backend is disk-RAM-bounded (base vectors demand-paged off the
  mmap); **HNSW** keeps its vectors in RAM (the `hnsw_rs` graph owns them), so true
  disk-resident approximate kNN (DiskANN-class) + GPU-native vector search are a
  future chapter, not shipped here.
- **No application consensus layer** — durability + replication is the NATS
  JetStream write-log; serving nodes are full replicas that tail it. The **columnar
  mmap segment disk tier** (RAM=hot/disk=all) + a local **RDB+AOF** is now
  default-compiled and runtime-selectable via `--persistence=segment` (default
  `cbor`); the old never-wired `storage_lsm` LSM backend and the `experimental`
  feature were removed.
- **K8s deployment** ships a real `Lumen` CRD + kube-rs reconcile loop, proven
  on a live kind cluster (`LUMEN_E2E_MODE=operator` kind-e2e: operator brings up
  the fleet + broker, survives serving-pod kill and broker kill with identical
  results), and is HA-safe via Lease leader-election (`replicas > 1` runs one
  active reconciler + standbys). This is a real operator baseline, not just YAML,
  but it is not yet the final production-hardening story: CRD validation,
  Kubernetes Conditions, TLS/NetworkPolicy/Ingress, observability parity, and
  upgrade/canary policy remain explicit deployment gaps below.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Search | - | implemented | auditing | conformance | not_ready | broad search evidence still mixes local gates with external perf/service gates |
| Lexical (BM25) | - | implemented | auditing | conformance | not_ready | WAND/block-max is an out-of-current-release future chapter |
| Exact & Filter (keyword / number / set) | - | implemented | auditing | conformance | not_ready | release proof remains tied to competitive perf evidence |
| Semantic & Perceptual (vector + hash) | 4141 | implemented | auditing | conformance | not_ready | DiskANN-class HNSW-on-disk remains future work |
| Hybrid (lexical + semantic fusion) | 4139 | implemented | auditing | conformance | not_ready | local conformance passes; production scope not selected |
| Duplicates | - | implemented | auditing | conformance | not_ready | local conformance passes; production scope not selected |
| Nested & Data-Table (group / has_child / collapse) | - | implemented | auditing | conformance | not_ready | local conformance passes; production scope not selected |
| Elastic Scale (columnar mmap disk tier — RAM=hot / disk=all) | - | implemented | auditing | conformance | not_ready | scale proof includes heavier/release evidence outside the default gate |
| Resilience & Log Replication | - | implemented | auditing | dogfood | not_ready | live NATS/kind dogfood gates are external-service dependent |
| Kubernetes-Native Deployment | - | implemented | auditing | conformance | not_ready | live operator e2e recency remains release-run dependent |
| HTTP / REST Integration | - | implemented | auditing | conformance | not_ready | runtime API proof remains outside the selected production scope |
| LLM / Agentic Integration (offline CLI — `spec` schema + `llm` topics) | 4143 | implemented | passing | conformance | ready | offline spec and llm CLI contract is covered by local spec_cli tests |
| Security & Auth | - | implemented | auditing | conformance | not_ready | bearer/RBAC e2e plus rustls peer config builder gate pass |
| Backup & Restore | - | implemented | auditing | conformance | not_ready | periodic snapshotter proof remains source-level |
| Observability | - | implemented | auditing | conformance | not_ready | OTLP service proof depends on the compose collector stack |
| Schema & Ops Lifecycle | - | implemented | auditing | conformance | not_ready | local conformance passes; production scope not selected |
| Operational characteristics (operability · speed · footprint · stability) | - | implemented | auditing | conformance | not_ready | operational proof remains tied to kind/perf/service evidence |

### Search

The product core, and the one big capability: lumen is a **pure search index** —
input a query (relevance + filters + sort), output ranked/sorted `external_id`s
only, never documents. A single **query planner** dispatches a per-shape
algorithm across every search flavor below — filter-as-pruning, early
termination, sort-by-field, selective-match-driver; boolean postings are roaring
bitmaps. The flavors of "find" are **sub-capabilities** of this one capability.

ID: search
Type: Service
Surfaces: HTTP: `POST /index` + `POST /search` - Client API on :7373 for indexing caller-owned records and querying ranked external_id results.; CLI: `lumen serve` - Starts the search service with configured persistence, API, and replication settings.
EC Dimensions: behavior: `rig` - request/query scenario conformance over the service API; efficiency: `rig + meter` - load pins plus resource attribution for service search workloads; security: `guard` - service API authorization and security findings gate; stability: `rig` - resilience scenarios for partition, packet loss, and recovery behavior
Efficiency Operating Point: local-vat-lumen-search-service
Efficiency Cube: projects/lumen/.aw/ec/efficiency/search-service.cube.json
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Input a query (relevance + filters + sort), output ranked/sorted `external_id`s only — never documents. One query planner dispatches per-shape algorithms (filter-as-pruning, early-termination, sort-by-field, selective-match-driver; roaring-bitmap postings) across all search-flavor sub-capabilities.
Gate Inventory:
- projects/lumen/tests/planner_diff.rs; projects/lumen/scripts/bench_vs_db.py

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Query planner & boolean eval (roaring postings) | epic | - | implemented | passing | conformance | projects/lumen/tests/planner_diff.rs |
| Filter + sort early-termination | epic | - | implemented | passing | conformance | projects/lumen/scripts/bench_vs_db.py (filter_sort, pure_sort) |
| selective-match-driver (drive cheapest positive incl. match) | epic | - | implemented | passing | conformance | projects/lumen/tests/collapse_nested.rs |
| Wide-range filter index (on-disk sorted-value range) | epic | - | implemented | passing | conformance | projects/lumen/src/storage.rs (segment_number_range_diff_tests); projects/lumen/tests/perf_gate_vs_db.rs (range cell) |
| Search p99 survives fault and recovers | epic | - | implemented | passing | dogfood | projects/lumen/tests/rig/cases/resilience |
| Graceful degradation under overload | epic | - | implemented | passing | dogfood | projects/lumen/tests/rig/cases/load; projects/lumen/tests/rig/config/pins |
| No fd socket thread leak | epic | - | implemented | passing | dogfood | projects/lumen/tests/rig/cases/endurance |
| No latency drift over soak | epic | - | implemented | passing | dogfood | projects/lumen/tests/rig/cases/endurance |

#### Efficiency - GENERATED (backfilled by `aw ec`; do not hand-edit)

Operating point: local-vat-lumen-search-service
Cube: projects/lumen/.aw/ec/efficiency/search-service.cube.json

### Lexical (BM25)

ID: search-lexical
Type: Service
Surfaces: HTTP: `POST /search` - `text` BM25 query surface over caller-owned `external_id`s.; CLI: `lumen serve` - Hosts the search API and tokenizer/analyzer-backed planner.
EC Dimensions: behavior: `cargo test -p lumen` - BM25 analyzer/ranking conformance; efficiency: `meter` - BM25 search profile through `projects/lumen/scripts/bench_vs_db.py`
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
BM25 ranking over `text`, with tokenization built in (`whitespace_lower` / `ngram` / `jieba`) — the one signal the caller does not pre-compute.
Gate Inventory:
- projects/lumen/scripts/bench_vs_db.py (text_bm25, text_and)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| BM25 ranking + analyzers (HashMap-free single-term / AND scoring fast path) | subepic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate_vs_db.rs (text_bm25, text_and); projects/lumen/src/storage.rs (segment_text_diff_tests — byte-identical disk BM25) |
| WAND / block-max early termination (skip non-competitive docs) | subepic | - | out_of_scope | none | none | future lexical lever for skewed tf / cold term corpora; the hot ranked-cache path now clears the 1M BM25 gate vs OpenSearch, but block-max is outside the current release scope |

### Exact & Filter (keyword / number / set)

ID: search-exact
Type: Service
Surfaces: HTTP: `POST /search` - `keyword`, `number`, and `set` predicates under boolean filters.; CLI: `lumen serve` - Hosts exact/filter query execution over the search API.
EC Dimensions: behavior: `cargo test -p lumen` - term/range/set planner conformance; efficiency: `meter` - filter and range search profile through `projects/lumen/scripts/bench_vs_db.py`
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
`keyword` term, `number` range, and `set` membership predicates; standalone predicates early-terminate; all compose under boolean and/or/not at roaring-bitmap speed.
Gate Inventory:
- projects/lumen/scripts/bench_vs_db.py (kw_term, range, bool_filter)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| term / range / set + early-termination | subepic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate_vs_db.rs (kw_term, range, bool_filter) |
| On-disk inverted (keyword/set) + sorted-value range (number) indexes — reopen drives from the mmap, RAM-bounded | subepic | - | implemented | passing | conformance | projects/lumen/src/storage.rs (segment_keyword/set_inverted_diff_tests, segment_number_range_diff_tests) |

### Semantic & Perceptual (vector + hash)

ID: search-vector
Type: Service
Surfaces: HTTP: `POST /search` - `vector` kNN, filtered kNN, and `hash` Hamming query surface.; CLI: `lumen serve` - Hosts vector/hash search execution over the search API.
EC Dimensions: behavior: `cargo test -p lumen` - vector/hash conformance; efficiency: `meter` - kNN and filtered-kNN competitive profile through `projects/lumen/scripts/bench_vs_db.py`
Root WI: 4141
Status: auditing
Required Verification: smoke, conformance
Promise:
Semantic kNN (`vector`; CPU HNSW + exact flat brute-force) and perceptual/structural `hash` (pHash / SimHash / b-bit MinHash) queried by Hamming distance. The caller owns all embeddings and hashes; lumen indexes the bits. kNN composes with filters **without recall collapse** (filter-correct kNN).
Gate Inventory:
- projects/lumen/tests/vector_e2e.rs; projects/lumen/tests/hash_hamming.rs; projects/lumen/tests/perf_gate_vs_db.rs (knn, filtered_knn vs pgvector); projects/lumen/scripts/bench_vs_db.py (knn)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| HNSW vector kNN (CPU) | subepic | - | implemented | passing | conformance | projects/lumen/tests/vector_e2e.rs |
| Filtered kNN — allow-list primitive (`search_knn_filtered`) | subepic | 4141 | implemented | passing | conformance | projects/lumen/src/vector_index.rs (filtered_knn_returns_nearest_within_allowlist_not_global_topk) |
| Filtered kNN — planner wiring (`knn AND filter`) + recall gate | subepic | 4142 | implemented | passing | conformance | projects/lumen/tests/vector_e2e.rs (filtered_knn_returns_nearest_within_filter_no_recall_collapse) |
| Competitive perf gate: `knn` + `filtered_knn` vs pgvector (opt-in `LUMEN_GATE_VECTOR=1`; OS host has no k-NN plugin) — `knn` is a TARGET (over-the-wire/real-corpus can lose), `filtered_knn` is a WIN (pgvector post-filters and collapses recall) | subepic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate_vs_db.rs (competitive_perf_gate: knn, filtered_knn); projects/lumen/tests/perf-baseline.json |
| flat-cpu vectors RAM-bounded on the disk tier (base rows demand-paged off the mmap, not re-materialized on reopen) | subepic | - | implemented | passing | conformance | projects/lumen/src/vector_index.rs (reopen_base_seg_plus_tail_plus_tombstone_equals_inram_oracle); projects/lumen/tests/disk_scale_proof.rs |
| HNSW graph on disk (DiskANN-class) — vectors stay in RAM in the graph; only flat-cpu is disk-RAM-bounded | subepic | - | out_of_scope | none | none | future GPU-native vector chapter outside the current release scope (hnsw_rs owns the vectors internally) |
| Hash / Hamming search (`hash` field + `hamming` query) | subepic | - | implemented | passing | conformance | projects/lumen/tests/hash_hamming.rs |

### Hybrid (lexical + semantic fusion)

ID: search-hybrid
Type: Service
Surfaces: HTTP: `POST /search` - RRF hybrid lexical+semantic query surface.; CLI: `lumen serve` - Hosts hybrid planner execution over the search API.
EC Dimensions: behavior: `cargo test -p lumen --test hybrid_rrf` - RRF fusion and planner conformance
Root WI: 4139
Status: auditing
Required Verification: conformance
Promise:
Fuse a lexical (BM25) ranking and a semantic (vector kNN) ranking into one result via Reciprocal Rank Fusion (RRF) — rank-based, so BM25 and cosine scales need no normalisation. Put filters inside each leg (`knn AND <filter>`) so the kNN leg stays filter-correct. This is the one retrieval an OLTP store cannot do at all.
Gate Inventory:
- projects/lumen/tests/hybrid_rrf.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| RRF fusion node (`rrf`) + planner integration | subepic | 4139 | implemented | passing | conformance | projects/lumen/tests/hybrid_rrf.rs |

### Duplicates

ID: search-duplicates
Type: Service
Surfaces: HTTP: `POST /search` - duplicate/group-by query surface returning matching `external_id`s.; CLI: `lumen serve` - Hosts duplicate search execution over the search API.
EC Dimensions: behavior: `cargo test -p lumen` - duplicate grouping and property conformance
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Find which `external_id`s share the same value — a search-flavor of group-by; bounded and posting-list-cheap. The primitive collapse-on-search builds on.
Gate Inventory:
- projects/lumen/tests/api_e2e.rs (duplicates_finds_groups); projects/lumen/tests/properties.rs (p6)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Duplicates group-by | subepic | - | implemented | passing | conformance | projects/lumen/tests/api_e2e.rs (duplicates_finds_groups) |

### Nested & Data-Table (group / has_child / collapse)

ID: search-nested
Type: Service
Surfaces: HTTP: `POST /search` - group, has_child, collapse, exists, duplicated, and CJK substring query surface.; CLI: `lumen serve` - Hosts nested/data-table query execution over the search API.
EC Dimensions: behavior: `cargo test -p lumen --test collapse_nested` - nested planner and data-table conformance
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Search Airtable-style data tables including nested `group` fields: group→child collection, a first-class `has_child` boolean clause, collapse-on-search, enum cascading paths (子母選單), and CJK substring. Correlation-correct (no cross-element false match). Plus `exists` (non-blank) and `duplicated` (collision) leaves that compose arbitrary presence/duplicate filters from the same boolean tree.
Gate Inventory:
- projects/lumen/tests/collapse_nested.rs; projects/lumen/scripts/bench_vs_db.py (group_nested)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| group→child mapping + collapse-on-search | subepic | - | implemented | passing | conformance | projects/lumen/tests/collapse_nested.rs |
| has_child boolean clause | subepic | - | implemented | passing | conformance | projects/lumen/tests/collapse_nested.rs (has_child_composes_in_boolean_tree) |
| enum level_match + CJK substring | subepic | - | implemented | passing | conformance | projects/lumen/tests/collapse_nested.rs (enum_path_and_level_match, ngram_cjk_substring) |
| `exists` / `duplicated` composite filter nodes (keyword/number/set; text/vector/hash rejected) | subepic | - | implemented | passing | conformance | projects/lumen/src/storage.rs (exists_filters_missing_field, exists_composes_with_boolean, duplicated_as_query_leaf, duplicated_composes_with_boolean, duplicated_min_group_size_floor_is_two) |

### Elastic Scale (columnar mmap disk tier — RAM=hot / disk=all)

ID: elastic-scale
Type: Service
Surfaces: CLI: `lumen serve --persistence=segment` - Runtime-selectable segment persistence mode.; Storage: columnar mmap segment files - RAM-hot/disk-all search tier.
EC Dimensions: behavior: `cargo test -p lumen` - segment reopen and byte-identical query conformance; efficiency: `meter` - disk-tier RSS and competitive search profile
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
**RAM is the hot working set; disk holds all the data.** Each field's bulk (forward payload + inverted index) is sealed into immutable, `applied_seq`-tagged, **columnar mmap segments**; the in-RAM driver is dropped on seal and queries demand-page off the mmap (a bounded `moka` decoded-posting cache is the inverted-index hot-zone). A collection far larger than RAM stays queryable — measured reopen+query resident growth is ~30-47% of full-in-RAM and **does not grow with N**. Per-field results stay byte-identical to the in-RAM path.
Gate Inventory:
- projects/lumen/tests/disk_scale_proof.rs; projects/lumen/docs/benchmarks-scale.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Columnar mmap segment engine (Number/Keyword/Set/Text/Hash/Vector) | epic | - | implemented | passing | conformance | projects/lumen/src/segment.rs (tests); projects/lumen/src/storage.rs (segment_predicate/keyword/set/number_range/text/vector_diff_tests, triple_path_diff_tests) |
| On-disk inverted/selective index — reopen does NOT rebuild the RAM driver (bound RAM) + per-field delete tombstone | epic | - | implemented | passing | conformance | projects/lumen/src/storage.rs (segment_keyword/set_inverted_diff_tests, segment_number_range_diff_tests, segment_text_diff_tests) |
| RAM=hot/disk=all bounded-RSS scale proof (demand-paged, flat in N) | epic | - | implemented | passing | conformance | projects/lumen/tests/disk_scale_proof.rs (scale_proof_reopen_rss_is_bounded) |
| Segment checkpoint (RDB) + local AOF durability (NATS-trim) | epic | - | implemented | passing | conformance | projects/lumen/src/storage.rs (checkpoint_engine_tests); projects/lumen/src/segment_rdb.rs; projects/lumen/src/aof.rs (crux_recovery_tests) |
| Bounded decoded-posting cache (warm-query hot-zone) + competitive disk gate | epic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate_vs_db.rs (competitive_perf_gate_disk) |
| Runtime selection: `--persistence=segment` (default `cbor`); segment engine default-compiled (the `experimental` feature + the never-wired `storage_lsm` backend were removed) | epic | - | implemented | passing | conformance | projects/lumen/src/bin/lumen.rs |

### Resilience & Log Replication

ID: resilience
Type: Service
Surfaces: CLI: `lumen serve` - NATS JetStream log tailing and serving-node rebuild path.; K8s: `projects/lumen/scripts/kind-e2e.sh` - broker/pod survival scenario.
EC Dimensions: stability: `rig` - broker kill, pod kill, and recovery behavior over the service deployment
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Writes publish to a NATS JetStream log; every serving node tails and folds it into its own index. Serving nodes are stateless and rebuild from the log + the caller. Survives broker kill and pod kill with byte-identical post-recovery results.
Gate Inventory:
- projects/lumen/scripts/kind-e2e.sh; projects/lumen/scripts/chaos.sh; projects/lumen/scripts/soak.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Log fan-out + rebuild-from-log | epic | - | implemented | passing | dogfood | projects/lumen/scripts/kind-e2e.sh |
| Broker-kill / pod-kill survival | epic | - | implemented | passing | dogfood | projects/lumen/scripts/chaos.sh; projects/lumen/scripts/soak.sh |

### Kubernetes-Native Deployment

ID: k8s-deployment
Type: Devops
Surfaces: K8s: `projects/lumen/k8s` - kustomize base, overlays, CRD, RBAC, and operator manifests.; Rust API: `lumen::operator` - kube-rs render/reconcile implementation.
EC Dimensions: behavior: `cargo test -p lumen --test operator_render` - manifest/render conformance; stability: `projects/lumen/scripts/kind-e2e.sh` - operator deploy and recovery dogfood gate
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Deploy declaratively two ways: a kustomize base + dev/staging/prod overlays, OR a `Lumen` CRD (`lumen.dev/v1alpha1`) reconciled by a kube-rs operator that renders + owns the serving Deployment/Service/ConfigMap/HPA/PDB/SA and the NATS broker, with `nats.externalUrl` to BYO. Cluster-agnostic base; e2e-gated on kind.
Gate Inventory:
- projects/lumen/k8s; projects/lumen/tests/operator_render.rs; projects/lumen/scripts/kind-e2e.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| kustomize base + overlays + HPA | epic | - | implemented | passing | conformance | projects/lumen/k8s |
| Lumen CRD + reconcile loop (kube-rs operator) | epic | - | implemented | passing | conformance | projects/lumen/src/operator; projects/lumen/tests/operator_render.rs |
| Operator kind-e2e deploy path | epic | - | implemented | passing | dogfood | projects/lumen/scripts/kind-e2e.sh (LUMEN_E2E_MODE=operator) |
| Leader-election HA (multi-replica operator) | epic | - | implemented | passing | conformance | projects/lumen/src/operator/lease.rs (election unit tests) |

**How to deploy — what lumen ships, and the few params you set.** lumen ships a
`Dockerfile` and a kustomize tree (`k8s/base` + overlays); you build + push the
image to your **own** registry and `kubectl apply -k` an overlay. There is no
published image (no Docker Hub) and no extra tool to install — kustomize is built
into `kubectl`. The handoff is **fill-in-the-blanks**, designed so a human *or an
agent* edits a copy of `k8s/overlays/template/` and self-checks with one grep:

```bash
# 1. build + push to your registry (e.g. Google Artifact Registry — no Docker Hub)
IMG=asia-east1-docker.pkg.dev/PROJECT/REPO/lumen:v1
docker build -t "$IMG" -f projects/lumen/Dockerfile .   # or: gcloud builds submit --tag "$IMG"
docker push "$IMG"

# 2. copy the template overlay and fill every REPLACE_ME__*
cp -r k8s/overlays/template k8s/overlays/myenv
#    edit k8s/overlays/myenv/kustomization.yaml

# 3. self-check — MUST print nothing before you apply (.example template skipped)
grep -rn REPLACE_ME k8s/overlays/myenv --include='*.yaml' | grep -v '\.example\.'

# 4. deploy
kubectl apply -k k8s/overlays/myenv
```

The required params all live in **your overlay copy**, never in `base`:

| Param | Where | Why it is required |
|---|---|---|
| image registry + tag | `images:` transformer | base ships `lumen:latest`; a cluster cannot pull an unprefixed name — point it at the registry you pushed to |
| `SHARD_COUNT` | ConfigMap patch | install-time crc32 client fan-out; **fixed for the cluster's life** — changing it after data exists re-routes every client and needs a rebuild |
| storage class | NATS `StatefulSet` patch | the one stateful component's PVC; template defaults to GKE SSD `premium-rwo` (balanced PD: `standard-rwo`, or delete the patch to use the cluster default) |
| auth + `LUMEN_TOKENS` | `secret.example.yaml` (optional) | off by default; to require bearer auth, copy the secret, fill tokens, and uncomment the auth block |

`k8s/overlays/{dev,staging,prod}` stay as worked references (real patch
examples); `template/` is the copy-to-customize blank. For BYO NATS, point
`LUMEN_NATS_URL` at your broker and drop the `nats-*` resources.

**Deployment footprint — what each path leaves on the cluster.** The two
declarative paths differ less in *what they run* than in the **cluster-scoped
footprint** they leave behind — the deciding factor when deploying into an
existing cluster where you do not hold cluster-admin. A `Lumen` deployment spans
four objects at three different scopes:

| Object | Scope | Notes |
|---|---|---|
| operator controller (`Deployment` + `ServiceAccount`) | namespaced | lives in `lumen-system` — an isolated namespace, but only the controller |
| `Lumen` **CRD** (the definition) | **cluster-scoped** | installed once, visible cluster-wide; K8s has no namespaced CRDs |
| operator **RBAC** (`ClusterRole` + `ClusterRoleBinding`) | **cluster-scoped** | cluster-wide watch of `Lumen` CRs (`k8s/operator/rbac.yaml`) |
| `Lumen` CR instance + serving children (`Deployment`/`Service`/`ConfigMap`/`HPA`/`PDB`/`SA`, NATS `StatefulSet`) | namespaced | the CR is `scope: Namespaced` — create it in any ns; children render in the same ns |

The `scope: Namespaced` line on the CRD describes the **CR instances** (you
create them in any namespace), *not* the CRD definition object — that is always
cluster-scoped. So the two paths have very different blast radii:

- **kustomize base + overlays is namespaced-only.** Every kind under
  `k8s/{base,overlays,components}` is namespaced except the `Namespace` it
  creates (`k8s/base/namespace.yaml` — drop it or point at an existing namespace
  for a shared cluster). **No CRD, no ClusterRole.**
- **CRD + operator adds two cluster-scoped objects** on top of the serving
  workload — the `Lumen` CRD and a `ClusterRole`/`ClusterRoleBinding` for
  cluster-wide CR watch — plus the controller in `lumen-system`. The controller
  is namespaced; the *global* footprint is the CRD + ClusterRole.

To deploy into an existing cluster with the smallest footprint, prefer the
namespaced-only kustomize path (a future offline `lumen k8s render` would emit
the same namespaced children from the same `render()` the operator uses, with no
cluster-scoped install). Reach for the operator when you want declarative
reconcile / drift-repair across many instances and can own the cluster-scoped
install.

**Known deployment hardening gaps:**

| Gap | Current State | Why It Matters |
|---|---|---|
| Live operator e2e recency | Scripted via `LUMEN_E2E_MODE=operator projects/lumen/scripts/kind-e2e.sh`; must be rerun on a machine with `kind` for each release. | Proves the CRD path still reconciles a real cluster, then survives serving-pod kill and broker kill. |
| CRD validation / immutability | Basic OpenAPI schema exists; stronger CEL-style invariants are not encoded yet. | Prevents unsafe live changes such as changing `shardCount`, invalid replica bounds, missing `tokensSecret` when auth is required, or undersized NATS storage. |
| Kubernetes Conditions | Status currently exposes phase, ready counts, shard count, `natsReady`, and message. | Production operators expect structured `Ready`, `Progressing`, `Degraded`, and `Reconciled` conditions with timestamps/reasons. |
| Observability parity | Kustomize prod/staging include the fuller ServiceMonitor + PrometheusRule bundle; operator render emits a smaller built-in observability set. | CRD users should get the same SLO alerts and scrape behavior as the hand-written prod overlay. |
| Network boundary / TLS / ingress | Auth is implemented; TLS binding is partial and not e2e-gated; NetworkPolicy/Ingress are not first-class CRD fields. | Production clusters need explicit traffic policy, TLS termination story, and ingress/service exposure controls. |
| Upgrade / rollout policy | Deployment uses rolling update, probes, HPA, and PDB; no explicit canary/version-skew policy is encoded. | Release operators need a safe story for image upgrades, CRD evolution, and broker/client compatibility. |

### HTTP / REST Integration

ID: rest-integration
Type: Service
Surfaces: HTTP: `:7373` REST API - HTTP/1.1 and h2c service endpoint.; HTTP: `/openapi.json` + `/docs` - OpenAPI and Swagger UI surfaces.
EC Dimensions: behavior: `cargo test -p lumen` - Axum route and OpenAPI conformance
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Plain HTTP/1.1 + HTTP/2 cleartext REST on `:7373`. Any REST client works with no driver, no wire protocol, and no connection-pool requirement; surface is described by OpenAPI 3 with a Swagger UI.
Gate Inventory:
- projects/lumen/src; projects/lumen/README.md (HTTP & clients, OpenAPI)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| REST API + HTTP/2 cleartext | epic | - | implemented | passing | conformance | projects/lumen/src (axum routes) |
| OpenAPI 3 + Swagger UI | epic | - | implemented | passing | smoke | /openapi.json, /docs |

### LLM / Agentic Integration (offline CLI — `spec` schema + `llm` topics)

A **subset of the `lumen` CLI** lets an agent discover the full request/response
format and schema **offline** — no running server, no network, no docs site. The
only requirement is that the `lumen` binary is installed; the binary is
self-describing via `lumen spec`:

- `lumen spec` / `lumen spec --format openapi-yaml` / `lumen spec --format
  json-schema` — the OpenAPI 3 document as JSON, the same OpenAPI as
  LLM-readable YAML, or just the request/response component schemas.
- `lumen spec --shapes` — a query-shape cookbook (ready-to-POST examples for
  every `QueryNode` variant plus sort / collapse).
- `lumen spec --fields` — the field-type / analyzer / vector-metric catalog.

All emit machine-readable JSON straight from `lumen::api::openapi()` and the
[`lumen::spec`] module with no server boot. (`lumen-openapi-dump` remains as a
back-compat alias for `lumen spec`.)

`lumen spec` answers *"what is the exact wire shape"*. **Agent-first DX** also
needs *"how do I string lumen into my system"* — so a sibling subset, `lumen
llm *`, emits focused **agent topics** an agent reads to go zero-to-integrated
offline:

- `lumen llm` / `lumen llm outline` — the topic map; use it to choose the
  smallest next topic instead of reading every page.
- `lumen llm workflow` — the product model (caller owns the source of truth;
  lumen is a derived index of `external_id`s, not a document store), the
  **declare schema → ingest via your own pub/sub into `POST /index` → search →
  hydrate from your store** workflow, a search-flavor decision map, connection
  (`:7373` + bearer), and the non-goals.
- `lumen llm integration` — the recommended Postgres/AlloyDB boundary:
  database commit + outbox/CDC first, external adapter-owned Pub/Sub retry/DLQ,
  HTTP writes into lumen, and no direct external publishing to lumen's NATS WAL.
- `lumen llm quickstart` — a minimal copy-paste end-to-end (create → index →
  search) as HTTP/curl.
- `lumen llm recipes` — task→query mappings (filtered kNN, dedupe, hybrid,
  nested `has_child`) as ready-to-POST bodies, consistent with `lumen spec
  --shapes`.

`lumen spec` is the schema; `lumen llm *` is the topic set. Together the binary
self-onboards an agent with no docs site and no running server.

ID: agentic-integration
Type: AgentFirst
Surfaces: CLI: `lumen spec` + `lumen spec --format openapi-yaml` + `lumen llm outline` + `lumen llm workflow` + `lumen llm integration` + `lumen llm quickstart` + `lumen llm recipes` - Offline self-description and agent onboarding commands that require no server, network, or docs site.
EC Dimensions: behavior: `cargo test -p lumen --test spec_cli` - offline schema and LLM topic conformance
Root WI: 4143
Status: verified
Required Verification: smoke, conformance
Promise:
An installed `lumen` binary self-onboards an agent **offline** (no server, no network): `lumen spec` emits the machine schema (OpenAPI JSON/YAML, JSON-schema, query-shape cookbook, field/analyzer catalog), and `lumen llm *` emits focused topics (outline, ingest→search→hydrate workflow, Postgres/AlloyDB integration boundary, quickstart, recipes, non-goals).
Gate Inventory:
- projects/lumen/tests/spec_cli.rs; projects/lumen/src/spec.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| `lumen spec` schema (OpenAPI JSON/YAML + JSON-schema, offline) | epic | - | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
| Query-shape cookbook + field/analyzer catalog | epic | - | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
| `lumen llm *` agent topics (outline / workflow / integration / quickstart / recipes) | epic | 4143 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |

### Security & Auth

ID: security-auth
Type: Service
Surfaces: HTTP: bearer-token auth and per-route RBAC on the REST API.; Peer transport config: `LUMEN_PEER_TLS_CERT` + `LUMEN_PEER_TLS_KEY` + `LUMEN_PEER_TLS_CA` + `LUMEN_PEER_MTLS` - rustls peer TLS material.
EC Dimensions: security: `cargo test -p lumen` - auth/RBAC denial matrix and rustls config construction
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Optional bearer-token auth (`LUMEN_AUTH=off` or `LUMEN_AUTH=required`) with per-token role-based authorization enforced on every API route; tokens supplied out-of-band via env/Secret. TLS (rustls) binding available.
Gate Inventory:
- projects/lumen/tests/auth_e2e.rs; projects/lumen/tests/authz_matrix_e2e.rs; projects/lumen/src/tls.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Bearer-token auth (`LUMEN_AUTH`) | epic | - | implemented | passing | conformance | projects/lumen/tests/auth_e2e.rs |
| Role-based authz matrix (per-route) | epic | - | implemented | passing | conformance | projects/lumen/tests/authz_matrix_e2e.rs |
| Adversarial query safety | epic | - | implemented | passing | conformance | projects/lumen/tests/coverage_gaps_e2e.rs (search_security_query_injection_rejects_bad_queries) |
| Score confidentiality | epic | - | implemented | passing | conformance | projects/lumen/tests/coverage_gaps_e2e.rs (search_security_result_leak_respects_collection_boundaries) |
| TLS (rustls) | epic | - | implemented | passing | smoke | `cargo test -p lumen tls`; projects/lumen/src/tls.rs (rustls server/client config builder) |

### Backup & Restore

ID: backup-restore
Type: Service
Surfaces: CLI: `lumen serve` - snapshot restore and periodic snapshot loop.; Rust API: `LocalFsRdbStore` - local snapshot sink implementation.
EC Dimensions: behavior: `cargo test -p lumen --test backup_restore_e2e` - snapshot/restore conformance
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
RDB snapshots to a pluggable sink as a cold-start baseline; a starting node restores the latest snapshot then tails the write log from that sequence — a bounded cold start instead of replaying the full log.
Gate Inventory:
- projects/lumen/tests/backup_restore_e2e.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| RDB snapshot + restore (LocalFsRdbStore) | epic | - | implemented | passing | conformance | projects/lumen/tests/backup_restore_e2e.rs |
| Periodic snapshotter (serve) | epic | - | implemented | passing | smoke | projects/lumen/src/bin/lumen.rs (snapshot loop) |

### Observability

ID: observability
Type: Devops
Surfaces: HTTP: `/metrics` - Prometheus text-format scrape endpoint.; K8s: ServiceMonitor + PrometheusRule manifests.; Config: `LUMEN_OTLP_ENDPOINT` - opt-in OTLP traces/metrics export.
EC Dimensions: behavior: `cargo test -p lumen` - metrics endpoint and observability wiring conformance
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
Prometheus text-format `/metrics` on the API port, a kustomize ServiceMonitor + PrometheusRule SLO alert bundle, structured json/pretty logs, and **opt-in OTLP export** (traces + metrics PUSHED to an OpenTelemetry collector via `LUMEN_OTLP_ENDPOINT`; the collector fans out to Prometheus/Jaeger, so a stateless replica fleet reports without per-pod scraping). `/metrics` pull stays for direct debug.
Gate Inventory:
- projects/lumen/tests/api_e2e.rs (/metrics); projects/lumen/k8s/components/observability; projects/lumen/compose.yaml (OTLP stack)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Prometheus `/metrics` endpoint | epic | - | implemented | passing | smoke | projects/lumen/tests/api_e2e.rs |
| ServiceMonitor + PrometheusRule | epic | - | implemented | passing | smoke | projects/lumen/k8s/components/observability |
| OTLP trace export (tower-http TraceLayer → tracing-opentelemetry → batch OTLP, opt-in; `otel` feature on in release builds) | epic | - | implemented | passing | conformance | projects/lumen/src/bin/lumen.rs (build_otel_tracer); projects/lumen/compose.yaml (Jaeger e2e: 13 request spans) |
| OTLP metrics push (observable instruments bridge the engine's atomic counters → PeriodicReader, no hot-path cost) | epic | - | implemented | passing | conformance | projects/lumen/src/bin/lumen.rs (init_otel_meter); projects/lumen/compose.yaml (Prometheus e2e: 11 metrics/replica) |

### Schema & Ops Lifecycle

ID: schema-ops
Type: Service
Surfaces: HTTP: collection DDL, drop-field, reindex, replay, stats, and metadata API routes.; CLI: `lumen serve` - Hosts schema/ops lifecycle endpoints.
EC Dimensions: behavior: `cargo test -p lumen` - schema DDL, drain, replay, stats, and metadata conformance
Root WI: -
Status: auditing
Required Verification: smoke, conformance
Promise:
The operational surface beyond search: collection DDL (create / add-field / drop-field), online drop-field drain, reindex/replay stream, and stats/metadata introspection.
Gate Inventory:
- projects/lumen/tests/drop_field_e2e.rs; projects/lumen/tests/reindex_stream_e2e.rs; projects/lumen/tests/stats_metadata_e2e.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Schema DDL + drop-field drain | epic | - | implemented | passing | conformance | projects/lumen/tests/drop_field_e2e.rs; projects/lumen/tests/drop_drain_e2e.rs |
| Reindex / replay stream | epic | - | implemented | passing | conformance | projects/lumen/tests/reindex_stream_e2e.rs |
| Stats + metadata | epic | - | implemented | passing | conformance | projects/lumen/tests/stats_metadata_e2e.rs |

### Operational characteristics (operability · speed · footprint · stability)

The capabilities above are *what* lumen finds. These are the **non-functional
promises** — and they are the real reason to run lumen next to an OLTP store
instead of bolting search onto the primary or standing up an ES cluster. They
are harder to state than a feature, so each is pinned to a concrete gate rather
than left as an adjective.

ID: ops-operability
Type: Devops
Surfaces: K8s: Deployment, HPA, PDB, and NATS manifests.; CLI: `lumen serve` - stateless serving process rebuilt from the log.; Bench harness: perf and disk-scale gates.
EC Dimensions: behavior: `projects/lumen/scripts/kind-e2e.sh` - operability dogfood gate; efficiency: `meter` - latency, throughput, RSS, and competitive regression profile; stability: `rig` - broker/pod recovery behavior
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Operate it without a DBA. Serving nodes are stateless cattle (`Deployment` + `HPA`, **no PVC**) with the index rebuilt from the log; the NATS broker is the only stateful component; there is **no consensus, leader election, or split-brain to run**; deploy declaratively via kustomize overlays or a `Lumen` CRD + operator. Search load lives on its own nodes — it never contends with the OLTP primary's CPU/RAM.
Gate Inventory:
- projects/lumen/scripts/kind-e2e.sh; projects/lumen/k8s; projects/lumen/src/operator

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Meta API: health / ready / metrics / version | epic | - | implemented | passing | conformance | projects/lumen/tests/api_e2e.rs |
| Stateless serving + rebuild-from-log (no PVC) | epic | - | implemented | passing | dogfood | projects/lumen/scripts/kind-e2e.sh |
| Perf-gate envelope (absolute latency + throughput floors) | epic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate.rs |
| Competitive regression gate (beat pg + OS per-cell, ratcheting) | epic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate_vs_db.rs; projects/lumen/tests/perf-baseline.json; all OS search cells and pg non-home-turf/native cells are WIN-gated |
| RAM=hot/disk=all columnar mmap segment tier + embedded single-node log | epic | - | implemented | passing | conformance | projects/lumen/tests/disk_scale_proof.rs; projects/lumen/src/storage.rs (checkpoint_engine_tests) |

**Stability (穩)** is the **Resilience & Log Replication** capability above: a
deployment survives broker kill and serving-pod kill with byte-identical
post-recovery results (`scripts/chaos.sh`, `scripts/soak.sh`), memory is bounded
because the bulk lives on mmap'd segments (RAM=hot/disk=all) demand-paged by the
kernel, and every node is a deterministic rebuild from the log.


### Non-goals (deliberate scope-out)

These are positioning decisions, not roadmap gaps. They define lumen by what it
**refuses** to be — the negative space is as much a promise as the capabilities
above. A feature request that crosses one of these lines is out of scope by
design, not by backlog.

- **Not an OLTP store / no transactions.** A write is atomic only at
  `(collection, external_id, field)` granularity — no multi-doc transactions, no
  MVCC, no serializable isolation. The **caller's source of truth** (Postgres /
  AlloyDB / MongoDB / S3 / …) owns the data and the transactions; lumen is a
  parallel derived index that converges after each write is published to the log
  and folded in by every serving node.
- **Not a document store.** lumen holds no original field values beyond what the
  inverted index needs — there is no `Document`, and a search hit is an
  `external_id` + score. Hydrating hits back to full records is the caller's job
  against its own store.
- **Not an analytics engine.** lumen does `find` + `count duplicates`. Group-by /
  bucket / metric / pipeline / histogram / percentile / cardinality aggregations
  are out of scope — pair lumen with an OLAP engine (ClickHouse / Druid /
  BigQuery / DuckDB) and dual-write. Engine *metadata* (collection size, per-field
  bytes, cache hit ratio, log-apply lag) is in scope at `/stats` and `/metrics` —
  that is introspection of lumen itself, not statistics over the caller's data.
- **Owns no models.** Vector and hash *indexing* are in scope; vector and hash
  *generation* are not. Callers send pre-computed `[f32; dim]` embeddings or
  fixed-bit hashes (CLIP / BGE / Whisper / pHash / NeuralHash / …); lumen owns the
  index and the distance math, never a model artefact.
- **Owns no ingestion.** Getting data in is the caller's own pub/sub into
  `POST /index` (CDC / logical replication / app writes). lumen bundles no
  connector, no Kafka source, no Beats/Logstash equivalent — but the DIY path is
  not a dead end: `examples/consumer_pg_logical.py` is a runnable reference
  (Postgres logical replication → crc32 shard route → `POST /index`).
- **No SQL surface, no joins, no subqueries.** The query is a typed boolean tree
  over relevance + filters + sort, not a query language.
- **No multi-region active-active.** A deployment (NATS broker + serving fleet) is
  single-region.

## Benchmarks

### Performance contract — enforced & ratcheting

Beating Postgres and OpenSearch on search is a **standing CI commitment, not a
one-time measurement**: `tests/perf_gate_vs_db.rs` drives lumen, Postgres
(`tokio-postgres`) and OpenSearch (`reqwest`) against one byte-identical corpus
and **fails the build** if lumen loses any *gated* search cell. The authoritative
thresholds live in **`tests/perf-baseline.json`**; full methodology, per-tier
numbers, resource columns, and reproduction live in
**[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md)**.

How the comparison stays honest (separate metrics, never conflated):

- **End-to-end, single-client** is the gated metric — lumen and OpenSearch share
  HTTP/JSON so the transport tax cancels. pg's binary wire beats HTTP/JSON on
  cheap btree point/range lookups on loopback, so those cells are **HTTP-EXEMPT**
  (annotated) and gated instead through a **native prepared-binary** path (Rust
  wire over Unix socket) — the cheap predicates still carry a hard floor.
- **Concurrent qps (10/100/1000)** and **write-path qps** are report-only by
  default; `LUMEN_PERF_STRICT=1` strict-gates the rows recorded in
  `perf-baseline.json`. Co-located CI keeps them report-only until CPU isolation;
  isolated-host repeats are the release-stable bar.

Each cell carries a threshold in `perf-baseline.json`: a **WIN cell** must hold
`max(1.0, 0.8 × recorded margin)` — a **ratchet**, so improving a cell locks the
new bar and it can only get better. **HTTP-EXEMPT cells** (pg btree lookups on
loopback) are separately gated by `pg_native` floors through the native path.
**Scale tiers:** 1K smoke/trend, **1M official competitive proof**; above 1M is
research-only.

**Current status — GREEN** (release, N=1M, in-memory + disk tier). Representative
serial search margins (full set, qps 10/100/1000 tiers, and history in
[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md) / `perf-baseline.json`):

| Cell | vs Postgres | vs OpenSearch (in-mem) | vs OpenSearch (disk) |
|---|---:|---:|---:|
| `text_bm25` | 815× | 4.5× | 23.0× |
| `text_and` | 96.9× | 7.7× | 10.9× |
| `filtered_search` | 61.4× | 7.3× | 4.6× |
| `filter_sort` | 43.9× | 4.1× | 6.0× |
| `pure_sort` | 83.6× | 3.9× | 5.2× |
| `kw_term` | EXEMPT¹ | 4.0× | 9.3× |
| `range` | EXEMPT¹ | 5.2× | 11.3× |
| `bool_filter` | EXEMPT¹ | 5.2× | 6.6× |

¹ pg cheap btree predicates are HTTP-EXEMPT; gated via the native prepared-binary
path — `kw_term` 6.2×, `range` 2.9×, `bool_filter` 39.6× vs pg prepared Unix socket.
Every OpenSearch cell holds a 3.0× WIN baseline (2.4× floor after the ratchet);
paced qps tiers stay ahead of OpenSearch on every WIN cell.

**Write path** — `tests/write_qps.rs` drives the real HTTP `POST /index`; treat
**JetStream as the standard write comparison** (embedded/local-sharded rows are
developer-loop trend checks). Latest 100-worker strict JetStream run: **8.5× vs
Postgres**, **3.4× vs OpenSearch**, 0 errors. `LUMEN_PERF_STRICT=1` strict-gates
the write margins when peer services are present; per-mode numbers and tuning
history live in `benchmarks-scale.md`.

### Footprint & stability

- **Index ~28.8 bytes/doc at 1M** — 5–7× smaller on disk than Postgres /
  OpenSearch; reported as a first-class disk-size metric alongside
  `pg_total_relation_size` and OpenSearch `_stats/store`.
- **RAM=hot/disk=all proven** (`tests/disk_scale_proof.rs`): a reopened
  collection's resident growth is ~30–47% of full-in-RAM and **does not grow with
  N** (forward payload demand-paged off the mmap).
- **Resident ~168 MB vs OpenSearch ~1.4 GB** (~8× smaller); tail p99
  `text_bm25` **1.0 ms** vs OpenSearch ~18 ms (no GC vs JVM pauses).
- **Stability:** 2M sustained searches held RSS flat with zero failed/errored/
  timed-out requests (Rust, no GC; mmap'd segments demand-paged by the kernel).

Full row-count × qps scaling, footprint tables, and the vs-pg / vs-OS breakdowns
live in **[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md)** (reproduce with
`./scripts/lumen_scale.sh`; rows above 1M are research-only).

## Data model

There are exactly three concepts on the wire:

| Concept       | What it is                                                |
|---------------|-----------------------------------------------------------|
| `Collection`  | A namespace + a schema (a map of field name → field type) |
| `Field`       | One typed column inside a collection                      |
| `external_id` | An opaque string chosen by the caller; lumen never mints it |

There is **no `Document`**. lumen does not store original field values
beyond what the inverted index needs to answer search and duplicate
queries. Hydrating search hits back to full records is the caller's
responsibility against its own store.

## Field types

Schema-first DDL. The declared `FieldType` deterministically picks the
index structure — there is no separate "index options" knob and no
auto-inference.

| FieldType | Index built on write                                                          | Query support              | Duplicate detection |
|-----------|-------------------------------------------------------------------------------|----------------------------|---------------------|
| `text`    | Tokenized inverted index (`token → sorted posting`); analyzer per field       | `match` (BM25, bag-of-words) | No                  |
| `keyword` | Exact inverted index (whole value as one term)                                | `term`, `terms`            | Yes                 |
| `number`  | Sorted inverted index (range-scannable)                                       | `term`, `range`            | Yes                 |
| `set`     | Multi-keyword (one posting per element)                                       | `term` (matches any element) | Yes (per element) |
| `vector`  | Dense `[f32; dim]` + ANN graph (HNSW CPU default; exact flat CPU brute-force) | `knn { vector, k }` with `cosine` / `dot` / `l2` metric | No |

Analyzers available for `text`: `jieba` (Chinese), `whitespace_lower`
(English / generic), `ngram` (configurable min/max). A field is bound
to one analyzer at declaration time.

A field cannot be both `text` and `keyword`. If both are needed (e.g.
"search by email substring *and* find duplicate emails"), declare two
fields and write twice — this keeps write amplification predictable.

## API surface

All endpoints are HTTP/2 JSON. The authoritative request / response
schemas are served by a running pod at `GET /openapi.json`. Offline
codegen pipes that spec out of the `lumen-openapi-dump` binary; see
[OpenAPI](#openapi) below.

### Schema (DDL)

```
PUT /collections/{id}
{
  "fields": {
    "bio":       { "type": "text",    "analyzer": "jieba" },
    "email":     { "type": "keyword" },
    "tags":      { "type": "keyword", "multi": true },
    "age":       { "type": "number" },
    "embedding": { "type": "vector",  "dim": 768, "metric": "cosine",
                   "backend": "hnsw-cpu", "quantize": "sq" }
  }
}
→ 200 { "collection_id": "users", "version": 1, "fields_count": 5 }
```

Online: adding a new field is immediate (postings start empty).
Re-declaring an existing field with the same spec is a no-op (PUT is
upsert-merge). Changing a field's type is rejected — drop the field
(`DELETE /collections/{id}/fields/{name}`) and re-add. `vector` field
configuration (`dim` / `metric` / `backend` / `quantize`) is immutable
for the field's lifetime.

### Index (write)

```
POST /collections/{id}/index
{
  "items": [
    { "external_id": "u_123", "field": "bio",   "value": "senior engineer in Taipei" },
    { "external_id": "u_123", "field": "email", "value": "a@x.com" },
    { "external_id": "u_123", "field": "tags",  "value": ["rust","db"] }
  ],
  "request_id": "..."        // optional, dedup TTL 5 min
}
→ 200 { "indexed": 3, "bytes_written": { "bio": 412, "email": 33, "tags": 88 }, "shard_lag_ms": 4 }
```

Re-writing `(external_id, field)` fully re-indexes that field. There
is no partial update.

### Delete

```
DELETE /collections/{id}/index/{external_id}             → 204    # all fields
DELETE /collections/{id}/index/{external_id}?field=bio   → 204    # one field
```

### Search

```
POST /collections/{id}/search
{
  "query": {
    "and": [
      { "match": { "field": "bio",  "text": "engineer taipei", "op": "and" } },
      { "term":  { "field": "tags", "value": "rust" } },
      { "range": { "field": "age",  "gte": 25, "lt": 40 } }
    ]
  },
  "limit": 20,
  "cursor": null
}
→ 200 {
  "hits": [
    { "external_id": "u_123", "score": 4.21 },
    { "external_id": "u_087", "score": 3.95 }
  ],
  "total": 217,        // estimate; ">10000" when truncated
  "cursor": "eyJvZmZzZXQiOjIwfQ==",
  "took_ms": 6
}
```

Search responses **only carry `external_id` + `score`** — never field
values. There is no `_source`.

**Pagination is keyset (search-after), depth-invariant.** The `cursor` is an
opaque token bound to the query that produced it: echo it back unchanged to
get the next page. For sorted (single number field) and score-ranked results
the token carries the LAST hit's position, so every page **seeks** —
O(log n) on the sorted index — instead of skipping; deep pages cost the same
as page 1 (measured at depth 50k over 100k docs: 86µs vs 28.7ms offset
skip). Stop when `cursor` is null. Legacy `{"offset":N}` tokens keep working
(O(offset) skip). Note: when continuing from a keyset cursor with
`track_total: true`, `total` counts the REMAINING matches from the cursor,
not the full set — read the full total off the first page.

### Duplicates

```
POST /collections/{id}/duplicates
{ "field": "email", "min_group_size": 2, "limit": 100 }
→ 200 {
  "groups": [
    { "value": "a@x.com", "external_ids": ["u_123","u_456","u_789"] },
    { "value": "b@y.com", "external_ids": ["u_201","u_990"] }
  ],
  "truncated": false,
  "took_ms": 12
}
```

`text` / `vector` fields do not support duplicates (semantics undefined).

### Exists / Duplicated (presence & collision filters)

Two query nodes for presence and collision. Both compose inside `and` / `or` /
`not` like any other leaf, so arbitrary combinations ("non-blank email **and**
duplicate phone") need no bespoke endpoint.

```
POST /collections/{id}/search
{
  "query": {
    "and": [
      { "exists":     { "field": "email" } },                      // email is non-blank
      { "duplicated": { "field": "phone", "min_group_size": 2 } }  // phone collides with another doc
    ]
  }
}
```

| Node | Matches |
|------|---------|
| `exists` | docs holding any value for `field`; `not exists` = "is empty" |
| `duplicated` | docs whose `field` value is shared by ≥ `min_group_size` docs (`min_group_size` defaults to / floors at 2) |

Both cover `keyword` / `number` / `set` fields. `text` / `vector` / `hash` are
rejected (presence/equality is undefined there — declare a `keyword` companion
field for a text "is empty" / duplicate filter).

`duplicated` vs the `/duplicates` endpoint: the endpoint returns *grouped*
results (`value → external_ids`) for an audit view; the `duplicated` query node
returns a *flat, composable* doc set you can intersect with other predicates in
one search.

### kNN (vector search)

```
POST /collections/{id}/search
{
  "query": {
    "knn": {
      "field": "embedding",
      "vector": [0.12, -0.04, ...],
      "k": 10
    }
  },
  "limit": 10
}
→ 200 {
  "hits": [
    { "external_id": "u_123", "score": 0.94 },
    { "external_id": "u_087", "score": 0.91 }
  ],
  "total": 10,
  "took_ms": 3
}
```

Scores are direction-normalised so higher = better regardless of
metric (`cosine` / `dot` use the raw similarity; `l2` reports
negated distance). `knn` can be composed inside `and` / `or` /
`not` with the other query nodes.

### Schema lifecycle

```
PUT    /collections/{id}                          # create or upsert-extend
DELETE /collections/{id}/fields/{field_name}      # online field drop
DELETE /collections/{id}                          # soft-delete (24h grace)
DELETE /collections/{id}?force=true               # immediate physical drop
GET    /collections                               # list (filtered by RBAC)
```

### Admin & ops

```
GET  /admin/backup                                # full SnapshotV1 JSON dump
POST /admin/restore                               # replace state from a snapshot
POST /admin/backup/local                          # snapshot → LocalFsSink (path + prefix)
GET  /debug/cluster                               # pod/shard/role/peers/replication-lag
GET  /metrics                                     # Prometheus text format
GET  /healthz                                     # liveness
GET  /readyz                                      # readiness (503 while draining)
GET  /openapi.json                                # live OpenAPI spec
GET  /docs                                        # Swagger UI (interactive "Try it out")
```

### Stats

Engine **metadata** about one collection. Per the v1 non-goals, this
describes the *index* (size, cardinality, cache health) — not the
caller's data. There are no aggregations here.

```
GET /collections/{id}/stats
→ 200 {
  "documents_indexed": 1234567,
  "fields": {
    "email": { "type": "keyword", "unique_terms": 1233110, "bytes": 40128830 },
    "bio":   { "type": "text",    "unique_terms": 482113,  "bytes": 32108920, "avg_doc_len": 28.4 },
    "age":   { "type": "number",  "unique_terms": 81,      "bytes": 9876543 }
  },
  "storage": { "total_bytes": 82114293 },
  "cache":   { "posting_hit_ratio": 0.87 },
  "last_indexed_at": "2026-05-28T16:42:11Z"
}
```

`last_indexed_at` is the typical "did my writes land?" probe — caller
writes N docs, then asserts `documents_indexed == N` and
`last_indexed_at` advanced. For Prometheus-shaped continuous
monitoring, `/metrics` carries the same numbers as gauges.

## HTTP & clients

The client API speaks **HTTP/1.1 and HTTP/2 cleartext (h2c) on the same
port** (`auto`) — the server accepts both, no flag needed. **HTTP/2 is the
recommended connection for serving**: one connection multiplexes many concurrent
streams, which is how lumen sustains its high-QPS search/index throughput. The
three setups, in order of preference:

- **Production (behind TLS) — HTTP/2 by default, for free.** An ingress / mesh
  terminating TLS negotiates h2 via ALPN, so every client gets it transparently.
  This is the recommended deployment.
- **Cleartext (dev / in-cluster) — h2c is opt-in.** h2c can't auto-negotiate (no
  ALPN), so a client must enable prior-knowledge (see table). A lumen connection
  *pool* over h2c is what the benchmark throughput numbers use.
- **Zero-driver fallback — plain HTTP/1.1 always works**, no special client:
  `requests`, `httpx`, `fetch`, `curl`, any REST client (lumen ships no client
  SDK — it's pure REST/OpenAPI; see `lumen llm`).

| Client | HTTP/1.1 | h2c (cleartext) opt-in | h2 over TLS (prod) |
|--------|----------|------------------------|--------------------|
| Python `requests` | ✅ | ✗ (no h2 support) | ✗ |
| Python `httpx` | ✅ | `pip install "httpx[http2]"` + `Client(http2=True)` | ✅ ALPN |
| `curl` | ✅ | `--http2-prior-knowledge` | `--http2` |
| Go `net/http` | ✅ | needs `x/net/http2` h2c transport | ✅ ALPN |
| browser (Swagger `/docs`) | ✅ | ✗ (browsers require TLS) | ✅ ALPN |

## OpenAPI

| Artefact              | When to use                                                  |
|-----------------------|--------------------------------------------------------------|
| `GET /openapi.json`   | Live spec from a running pod — codegen against an actual env |
| `GET /docs`           | Interactive Swagger UI ("Try it out")                        |
| `lumen-openapi-dump`  | Offline dump — codegen / review without a running server     |

The dump binary generates from the same Rust code as the live endpoint
(`#[derive(utoipa::OpenApi)]` on `api::ApiDoc`):

```bash
cargo run -q -p lumen --bin lumen-openapi-dump > /tmp/lumen-openapi.json
```

Pipe that into your codegen tool of choice. There is no in-tree
snapshot — the live endpoint and the dump binary are the single source
of truth.
