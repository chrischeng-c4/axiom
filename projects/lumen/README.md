# lumen

A K8s-native, **log-replicated search specialist**. Five flavors of
"find":

- **Exact** — `keyword` / `number` / `set`
- **Lexical** — `text` (BM25, with tokenize built in)
- **Semantic** — `vector` (HNSW; optional wgpu GPU kNN)
- **Perceptual / structural** — `hash` (pHash / SimHash / b-bit MinHash, Hamming distance)
- **Duplicates** — find which `external_id`s share the same value (a search-flavor of group-by; bounded, posting-list-cheap)

The caller owns the representation:

- Embeddings? **Caller** runs CLIP / BGE / Whisper / VideoMAE; lumen never owns a model artefact.
- Perceptual hashes? **Caller** runs `imagehash` / `datasketch`; lumen indexes the bits.
- Lexical tokenization? **lumen** does it — that's the one place caller doesn't compute (`whitespace_lower` / `ngram` / `jieba`).

The caller also owns the **source of truth**: lumen is a parallel
derived index, not an OLTP store and not an analytics engine.

- **Caller owns the source of truth**. lumen is a parallel index, never
  the system of record. Documents are *not* a lumen concept — only the
  caller's `external_id` is.
- **Log-driven, derived, rebuildable**. A write is *published to a log*,
  not applied where it lands; every serving node tails the log and folds
  it into its own index. Lossable but rebuildable from the log + the
  caller.
- **Client API on `:7373`** (HTTP/1.1 + HTTP/2 cleartext — REST clients
  need nothing special; see [HTTP](#http--clients)).
- **Sharded**: `crc32(collection_id) % shard_count` routes on the client.
  Shard count is install-time, not online-changeable.

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

**Honest scope (do not over-claim):**

- **Ingestion is the caller's own pub/sub** into `POST /index` (CDC / logical
  replication / app writes). lumen bundles no connector and owns no upstream
  subscription — it is a parallel derived index, rebuildable from the source + log.
- **Lexical ranking at scale** has no WAND / block-max yet, so pure unfiltered
  single-term ranking trails an ES-class peer at high scale.
- **Wide-range filters** trail a BKD points-tree peer; no segment range index yet.
- **GPU (`wgpu`) kNN** is experimental and not exercised in CI; CPU HNSW is the
  proven path.
- **No application consensus layer** — durability + replication is the NATS
  JetStream write-log; serving nodes are full replicas that tail it. The LSM disk
  backend (`storage_lsm`) is the one subsystem still behind the `experimental`
  feature, pending its promotion to a runtime-selectable tier.
- **K8s deployment** ships a real `Lumen` CRD + kube-rs reconcile loop, proven
  on a live kind cluster (`LUMEN_E2E_MODE=operator` kind-e2e: operator brings up
  the fleet + broker, survives serving-pod kill and broker kill with identical
  results), and is HA-safe via Lease leader-election (`replicas > 1` runs one
  active reconciler + standbys).

### Search

The product core, and the one big capability: lumen is a **pure search index** —
input a query (relevance + filters + sort), output ranked/sorted `external_id`s
only, never documents. A single **query planner** dispatches a per-shape
algorithm across every search flavor below — filter-as-pruning, early
termination, sort-by-field, selective-match-driver; boolean postings are roaring
bitmaps. The flavors of "find" are **sub-capabilities** of this one capability.

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| search | - | auditing | Input a query (relevance + filters + sort), output ranked/sorted `external_id`s only — never documents. One query planner dispatches per-shape algorithms (filter-as-pruning, early-termination, sort-by-field, selective-match-driver; roaring-bitmap postings) across all search-flavor sub-capabilities. | smoke, conformance | projects/lumen/tests/planner_diff.rs; projects/lumen/scripts/bench_vs_db.py |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Query planner & boolean eval (roaring postings) | epic | - | implemented | passing | conformance | projects/lumen/tests/planner_diff.rs |
| Filter + sort early-termination | epic | - | implemented | passing | conformance | projects/lumen/scripts/bench_vs_db.py (filter_sort, pure_sort) |
| selective-match-driver (drive cheapest positive incl. match) | epic | - | implemented | passing | conformance | projects/lumen/tests/collapse_nested.rs |
| Wide-range filter index (BKD) | epic | - | planned | none | none | - |

#### Lexical (BM25)

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| search-lexical | - | auditing | BM25 ranking over `text`, with tokenization built in (`whitespace_lower` / `ngram` / `jieba`) — the one signal the caller does not pre-compute. | smoke, conformance | projects/lumen/scripts/bench_vs_db.py (text_bm25, text_and) |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| BM25 ranking + analyzers | subepic | - | partial | passing | conformance | projects/lumen/scripts/bench_vs_db.py (text_bm25); WAND/block-max not yet implemented |

#### Exact & Filter (keyword / number / set)

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| search-exact | - | auditing | `keyword` term, `number` range, and `set` membership predicates; standalone predicates early-terminate; all compose under boolean and/or/not at roaring-bitmap speed. | smoke, conformance | projects/lumen/scripts/bench_vs_db.py (kw_term, range, bool_filter) |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| term / range / set + early-termination | subepic | - | implemented | passing | conformance | projects/lumen/scripts/bench_vs_db.py (kw_term, range) |

#### Semantic & Perceptual (vector + hash)

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| search-vector | - | auditing | Semantic kNN (`vector`; HNSW, optional `wgpu` GPU) and perceptual/structural `hash` (pHash / SimHash / b-bit MinHash) queried by Hamming distance. The caller owns all embeddings and hashes; lumen indexes the bits. kNN composes with filters **without recall collapse** (filter-correct kNN). | smoke, conformance | projects/lumen/tests/vector_e2e.rs; projects/lumen/tests/hash_hamming.rs; projects/lumen/scripts/bench_vs_db.py (knn) |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| HNSW vector kNN (CPU) | subepic | - | implemented | passing | conformance | projects/lumen/tests/vector_e2e.rs |
| Filtered kNN — allow-list primitive (`search_knn_filtered`) | subepic | 4141 | implemented | passing | conformance | projects/lumen/src/vector_index.rs (filtered_knn_returns_nearest_within_allowlist_not_global_topk) |
| Filtered kNN — planner wiring (`knn AND filter`) + recall gate | subepic | 4142 | implemented | passing | conformance | projects/lumen/tests/vector_e2e.rs (filtered_knn_returns_nearest_within_filter_no_recall_collapse) |
| GPU kNN (wgpu) | subepic | - | partial | planned | smoke | experimental; not exercised in CI |
| Hash / Hamming search (`hash` field + `hamming` query) | subepic | - | implemented | passing | conformance | projects/lumen/tests/hash_hamming.rs |

#### Hybrid (lexical + semantic fusion)

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| search-hybrid | 4139 | auditing | Fuse a lexical (BM25) ranking and a semantic (vector kNN) ranking into one result via Reciprocal Rank Fusion (RRF) — rank-based, so BM25 and cosine scales need no normalisation. Put filters inside each leg (`knn AND <filter>`) so the kNN leg stays filter-correct. This is the one retrieval an OLTP store cannot do at all. | conformance | projects/lumen/tests/hybrid_rrf.rs |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| RRF fusion node (`rrf`) + planner integration | subepic | 4139 | implemented | passing | conformance | projects/lumen/tests/hybrid_rrf.rs |

#### Duplicates

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| search-duplicates | - | auditing | Find which `external_id`s share the same value — a search-flavor of group-by; bounded and posting-list-cheap. The primitive collapse-on-search builds on. | smoke, conformance | projects/lumen/tests/api_e2e.rs (duplicates_finds_groups); projects/lumen/tests/properties.rs (p6) |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Duplicates group-by | subepic | - | implemented | passing | conformance | projects/lumen/tests/api_e2e.rs (duplicates_finds_groups) |

#### Nested & Data-Table (group / has_child / collapse)

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| search-nested | - | auditing | Search Airtable-style data tables including nested `group` fields: group→child collection, a first-class `has_child` boolean clause, collapse-on-search, enum cascading paths (子母選單), and CJK substring. Correlation-correct (no cross-element false match). | smoke, conformance | projects/lumen/tests/collapse_nested.rs; projects/lumen/scripts/bench_vs_db.py (group_nested) |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| group→child mapping + collapse-on-search | subepic | - | implemented | passing | conformance | projects/lumen/tests/collapse_nested.rs |
| has_child boolean clause | subepic | - | implemented | passing | conformance | projects/lumen/tests/collapse_nested.rs (has_child_composes_in_boolean_tree) |
| enum level_match + CJK substring | subepic | - | implemented | passing | conformance | projects/lumen/tests/collapse_nested.rs (enum_path_and_level_match, ngram_cjk_substring) |

### Elastic Scale (collection-LRU)

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| elastic-scale | - | auditing | RAM is a working set: idle collections snapshot to disk and restore on demand (collection-LRU), so hot tables run at full in-memory speed while dataset and vector scale are bounded by cheap disk, not RAM. | smoke, conformance | projects/lumen/tests/collection_lru.rs |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| collection-LRU evict / restore | epic | - | implemented | passing | conformance | projects/lumen/tests/collection_lru.rs |
| LRU budget enforcement + thread-safe restore | epic | - | implemented | passing | conformance | projects/lumen/tests/collection_lru.rs (budget_enforced_and_lru, concurrent_restore_thread_safe) |
| has_child against an evicted child collection | epic | - | implemented | passing | conformance | projects/lumen/tests/collection_lru.rs (has_child_restores_evicted_child_collection) |
| LSM disk backend (`storage_lsm`, experimental feature, unwired) | epic | - | out_of_scope | none | none | - |

### Resilience & Log Replication

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| resilience | - | auditing | Writes publish to a NATS JetStream log; every serving node tails and folds it into its own index. Serving nodes are stateless and rebuild from the log + the caller. Survives broker kill and pod kill with byte-identical post-recovery results. | conformance, dogfood | projects/lumen/scripts/kind-e2e.sh; projects/lumen/scripts/chaos.sh; projects/lumen/scripts/soak.sh |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Log fan-out + rebuild-from-log | epic | - | implemented | passing | dogfood | projects/lumen/scripts/kind-e2e.sh |
| Broker-kill / pod-kill survival | epic | - | implemented | passing | dogfood | projects/lumen/scripts/chaos.sh; projects/lumen/scripts/soak.sh |

### Kubernetes-Native Deployment

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| k8s-deployment | - | auditing | Deploy declaratively two ways: a kustomize base + dev/staging/prod overlays, OR a `Lumen` CRD (`lumen.dev/v1alpha1`) reconciled by a kube-rs operator that renders + owns the serving Deployment/Service/ConfigMap/HPA/PDB/SA and the NATS broker, with `nats.externalUrl` to BYO. Cluster-agnostic base; e2e-gated on kind. | smoke, conformance | projects/lumen/k8s; projects/lumen/tests/operator_render.rs; projects/lumen/scripts/kind-e2e.sh |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| kustomize base + overlays + HPA | epic | - | implemented | passing | conformance | projects/lumen/k8s |
| Lumen CRD + reconcile loop (kube-rs operator) | epic | - | implemented | passing | conformance | projects/lumen/src/operator; projects/lumen/tests/operator_render.rs |
| Operator kind-e2e deploy path | epic | - | implemented | passing | dogfood | projects/lumen/scripts/kind-e2e.sh (LUMEN_E2E_MODE=operator) |
| Leader-election HA (multi-replica operator) | epic | - | implemented | passing | conformance | projects/lumen/src/operator/lease.rs (election unit tests) |

### HTTP / REST Integration

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| rest-integration | - | auditing | Plain HTTP/1.1 + HTTP/2 cleartext REST on `:7373`. Any REST client works with no driver, no wire protocol, and no connection-pool requirement; surface is described by OpenAPI 3 with a Swagger UI. | smoke, conformance | projects/lumen/src; projects/lumen/README.md (HTTP & clients, OpenAPI) |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| REST API + HTTP/2 cleartext | epic | - | implemented | passing | conformance | projects/lumen/src (axum routes) |
| OpenAPI 3 + Swagger UI | epic | - | implemented | passing | smoke | /openapi.json, /docs |

### LLM / Agentic Integration (offline CLI — `spec` schema + `llm` playbook)

A **subset of the `lumen` CLI** lets an agent discover the full request/response
format and schema **offline** — no running server, no network, no docs site. The
only requirement is that the `lumen` binary is installed; the binary is
self-describing via `lumen spec`:

- `lumen spec` / `lumen spec --format json-schema` — the OpenAPI 3 document, or
  just the request/response component schemas.
- `lumen spec --shapes` — a query-shape cookbook (ready-to-POST examples for
  every `QueryNode` variant plus sort / collapse).
- `lumen spec --fields` — the field-type / analyzer / vector-metric catalog.

All emit machine-readable JSON straight from `lumen::api::openapi()` and the
[`lumen::spec`] module with no server boot. (`lumen-openapi-dump` remains as a
back-compat alias for `lumen spec`.)

`lumen spec` answers *"what is the exact wire shape"*. **Agent-first DX** also
needs *"how do I string lumen into my system"* — so a sibling subset, `lumen
llm *`, emits the **integration playbook** an agent reads to go zero-to-integrated
offline:

- `lumen llm` / `lumen llm guide` — the integration playbook: the mental model
  (caller owns the source of truth; lumen is a derived index of `external_id`s,
  not a document store), the **declare schema → ingest via your own pub/sub into
  `POST /index` → search → hydrate from your store** workflow, a search-flavor
  decision guide, connection (`:7373` + bearer), and the non-goals.
- `lumen llm quickstart` — a minimal copy-paste end-to-end (create → index →
  search) as HTTP/curl.
- `lumen llm recipes` — task→query mappings (filtered kNN, dedupe, hybrid,
  nested `has_child`) as ready-to-POST bodies, consistent with `lumen spec
  --shapes`.

`lumen spec` is the schema; `lumen llm` is the playbook. Together the binary
self-onboards an agent with no docs site and no running server.

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| agentic-integration | - | auditing | An installed `lumen` binary self-onboards an agent **offline** (no server, no network): `lumen spec` emits the machine schema (OpenAPI / JSON-schema, query-shape cookbook, field/analyzer catalog), and `lumen llm *` emits the agent integration playbook (mental model, ingest→search→hydrate workflow, flavor-decision guide, recipes, non-goals). | smoke, conformance | projects/lumen/tests/spec_cli.rs; projects/lumen/src/spec.rs |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| `lumen spec` schema (OpenAPI + JSON-schema, offline) | epic | - | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
| Query-shape cookbook + field/analyzer catalog | epic | - | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs |
| `lumen llm *` agent integration playbook (guide / quickstart / recipes) | epic | 4143 | implemented | passing | conformance | projects/lumen/tests/spec_cli.rs (llm_guide / llm_quickstart / llm_recipes) |

### Security & Auth

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| security-auth | - | auditing | Optional bearer-token auth (`LUMEN_AUTH=off\|required`) with per-token role-based authorization enforced on every API route; tokens supplied out-of-band via env/Secret. TLS (rustls) binding available. | smoke, conformance | projects/lumen/tests/auth_e2e.rs; projects/lumen/tests/authz_matrix_e2e.rs |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Bearer-token auth (`LUMEN_AUTH`) | epic | - | implemented | passing | conformance | projects/lumen/tests/auth_e2e.rs |
| Role-based authz matrix (per-route) | epic | - | implemented | passing | conformance | projects/lumen/tests/authz_matrix_e2e.rs |
| TLS (rustls) | epic | - | partial | planned | smoke | projects/lumen/src/tls.rs (binding; not e2e-gated) |

### Backup & Restore

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| backup-restore | - | auditing | RDB snapshots to a pluggable sink as a cold-start baseline; a starting node restores the latest snapshot then tails the write log from that sequence — a bounded cold start instead of replaying the full log. | smoke, conformance | projects/lumen/tests/backup_restore_e2e.rs |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| RDB snapshot + restore (LocalFsRdbStore) | epic | - | implemented | passing | conformance | projects/lumen/tests/backup_restore_e2e.rs |
| Periodic snapshotter (serve) | epic | - | implemented | passing | smoke | projects/lumen/src/bin/lumen.rs (snapshot loop) |

### Observability

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| observability | - | auditing | Prometheus text-format `/metrics` on the API port, a kustomize ServiceMonitor + PrometheusRule SLO alert bundle, and structured json/pretty logs. | smoke | projects/lumen/tests/api_e2e.rs (/metrics); projects/lumen/k8s/components/observability |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Prometheus `/metrics` endpoint | epic | - | implemented | passing | smoke | projects/lumen/tests/api_e2e.rs |
| ServiceMonitor + PrometheusRule | epic | - | implemented | passing | smoke | projects/lumen/k8s/components/observability |

### Schema & Ops Lifecycle

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| schema-ops | - | auditing | The operational surface beyond search: collection DDL (create / add-field / drop-field), online drop-field drain, reindex/replay stream, and stats/metadata introspection. | smoke, conformance | projects/lumen/tests/drop_field_e2e.rs; projects/lumen/tests/reindex_stream_e2e.rs; projects/lumen/tests/stats_metadata_e2e.rs |

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

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| ops-operability | - | auditing | Operate it without a DBA. Serving nodes are stateless cattle (`Deployment` + `HPA`, **no PVC**) with the index rebuilt from the log; the NATS broker is the only stateful component; there is **no consensus, leader election, or split-brain to run**; deploy declaratively via kustomize overlays or a `Lumen` CRD + operator. Search load lives on its own nodes — it never contends with the OLTP primary's CPU/RAM. | conformance, dogfood | projects/lumen/scripts/kind-e2e.sh; projects/lumen/k8s; projects/lumen/src/operator |
| ops-speed | - | auditing | Low-latency search from an early-terminating planner over roaring-bitmap postings, in-memory serving, and HNSW kNN — no GC pauses (Rust). Perf-gate envelope on a dev box: `term` < 20 ms, BM25 `match` over 10k docs < 50 ms, 5k keyword writes < 1 s. | conformance | projects/lumen/tests/perf_gate.rs; projects/lumen/scripts/bench_vs_db.py |
| ops-footprint | - | auditing | Lightweight: a single Rust binary, **no JVM**. RAM is a working set — idle collections snapshot to disk (collection-LRU), so dataset and vector scale are bounded by cheap disk, not RAM. A single node runs on an embedded in-process log with no broker at all. | conformance | projects/lumen/tests/collection_lru.rs |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Stateless serving + rebuild-from-log (no PVC) | epic | - | implemented | passing | dogfood | projects/lumen/scripts/kind-e2e.sh |
| Perf-gate envelope (latency + throughput floors) | epic | - | implemented | passing | conformance | projects/lumen/tests/perf_gate.rs |
| RAM-as-working-set (collection-LRU) + embedded single-node log | epic | - | implemented | passing | conformance | projects/lumen/tests/collection_lru.rs |

**Stability (穩)** is the **Resilience & Log Replication** capability above: a
deployment survives broker kill and serving-pod kill with byte-identical
post-recovery results (`scripts/chaos.sh`, `scripts/soak.sh`), memory is bounded
by the LRU budget, and every node is a deterministic rebuild from the log.

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

Indicative head-to-head on one machine — a single seeded 100k-doc corpus loaded
byte-identically into every engine, warm. Latency is `bench_vs_db.py` (client
min, warm persistent connection); throughput is `h2load` for lumen/OpenSearch
and `pgbench` (prepared, persistent — Postgres's best case) for PG. **Numbers are
illustrative of the shape, not a certified SLA** — reproduce with
`scripts/bench_vs_db.py`.

**vs PostgreSQL — throughput (the workload that matters):** lumen serves search
over an h2c connection *pool* (a few connections, many multiplexed streams).

| Query | lumen | PostgreSQL | lumen advantage |
|---|---:|---:|---:|
| `kw_term` (keyword) | 219k req/s | 64k tps | **3.4×** |
| `filtered_search` (BM25 + filter) | 74k req/s | 8.7k tps | **8.4×** |
| `text_bm25` | 38k req/s | 2.8k tps | **13×** |
| `group_nested_count` (no early-stop in PG) | — | — | **40×** |

Single trivial point-lookups (one `term`/`range`, tiny result) are the one place
PG's binary wire protocol beats lumen's HTTP+JSON on *per-request* latency — a
protocol tax, not an engine gap (the throughput above is the engine truth). At
volume lumen wins across the board.

**vs OpenSearch — same HTTP/JSON protocol, so protocol overhead cancels:**

| Dimension | lumen | OpenSearch | lumen advantage |
|---|---:|---:|---:|
| Search latency (per cell, min) | 0.47–0.77 ms | 1.06–1.94 ms | **2.3–3.1× faster** |
| Tail p99 (`text_bm25`) | **1.0 ms** | 18 ms | no GC vs JVM GC pauses |
| Throughput (`filtered_search`, HTTP/1.1) | **62k req/s** | 17k req/s | **3.6×** |
| Resident memory | **168 MB** | 1.4 GB | **8.4× smaller** |

lumen beats an ES-class engine on latency, tail, throughput, and footprint at
once — because it does far less (pure index, no documents, no JVM).

**Stability:** 2M sustained searches held RSS flat at ~170 MB with zero failed /
errored / timed-out requests (Rust, no GC; bounded `moka` + collection-LRU
caches).

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
| `vector`  | Dense `[f32; dim]` + ANN graph (HNSW CPU default; `gpu` feature → wgpu brute-force kNN) | `knn { vector, k }` with `cosine` / `dot` / `l2` metric | No |

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
port** (`auto`). For a REST/JSON API, **HTTP/1.1 needs nothing special**
— `requests`, `httpx` (default), `fetch`, `curl`, the Rust client all
just work. HTTP/2 is an opt-in throughput optimization:

| Client | HTTP/1.1 (default) | h2c (cleartext) opt-in | h2 over TLS (prod) |
|--------|--------------------|------------------------|--------------------|
| Python `requests` | ✅ | ✗ (no h2 support) | ✗ |
| Python `httpx` | ✅ | `pip install "httpx[http2]"` + `Client(http2=True)` | ✅ ALPN |
| `curl` | ✅ | `--http2-prior-knowledge` | `--http2` |
| Go `net/http` | ✅ | needs `x/net/http2` h2c transport | ✅ ALPN |
| browser (Swagger `/docs`) | ✅ | ✗ (browsers require TLS) | ✅ ALPN |

In production behind TLS (ingress / mesh terminating TLS), HTTP/2 is
negotiated transparently via ALPN — every client gets it for free.

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

