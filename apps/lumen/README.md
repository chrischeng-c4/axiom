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
  need nothing special; high-throughput clients should use HTTP/2 multiplexing;
  see [HTTP](#http--clients)).
- **Sharded**: `hash(collection_id, routing_key || external_id)` selects a
  virtual bucket, and a versioned operator-owned shard map assigns buckets to
  physical storage shards. `shardCount` controls storage ownership,
  `replicasPerShard` controls HA/raft quorum per shard, and HPA never changes
  data ownership.
- **Agent-first offline integration surface**: `lumen spec` emits the exact
  machine schema, including `lumen spec --format openapi-yaml` for LLM-readable
  OpenAPI, while `lumen llm --topic outline --format json` emits a typed
  `cclab.llm.v2` task manifest and `lumen llm --topic <id>` emits the smallest
  source-backed runbook needed to wire Lumen into an app without a docs site or
  running server.

## Contributing

Project-local authoring and verification rules live in
[`CONTRIBUTING.md`](CONTRIBUTING.md). Repository-wide rules remain
authoritative when the two differ.

## Capability Contract

Lumen is a derived-index service. Its core job is to build indexes over
caller-owned data and query those indexes. Lumen is not a system of record,
analytics engine, identity provider, or certificate authority.

The canonical contract is [`CAPABILITIES.md`](CAPABILITIES.md). It has two
feature roots only. Core means the capability changes what Lumen indexes or
how it answers queries. Non-core means the capability makes those jobs usable
in production; it does not mean optional.

### Core Features

- **Indexing** — schema validation, ingestion, mutation, derived-index
  persistence, checkpointing, and rebuild over caller-owned data.
- **Querying** — lexical, exact/filter, vector/hash, hybrid, duplicate/nested,
  pagination, sort, and read-consistency semantics.

### Non-Core Features

- **Kubernetes-Native Deployment** — independently rendered image, CRD,
  operator, and instance layers with declarative reconciliation.
- **Security & Access** — Kubernetes ServiceAccount TokenReview/SAR for client
  requests plus separate instance-scoped mTLS for Raft peers. This capability
  is being replaced and is not production-ready.
- **Scaling & Availability** — elastic segments, shard topology, replicas,
  failover, and replacement bootstrap.
- **Durability & Recovery** — WAL/checkpoint recovery, backup/restore, and cold
  seed.
- **Operations & Observability** — health, readiness, conditions, metrics,
  events, alerts, tracing, and long-running-operation state.
- **API, CLI & Agent Integration** — HTTP/1.1 and HTTP/2, OpenAPI, clients,
  standard CLI surfaces, chainable output, and offline agent guidance.

Stable capability and claim IDs live in `CAPABILITIES.md`. Delivery planning
lives in GitHub and references those IDs one way.

## Benchmarks

### Performance contract — enforced & ratcheting

Beating Postgres and OpenSearch on search is a **standing CI commitment, not a
one-time measurement**: `tests/perf_gate_vs_db.rs` drives lumen, Postgres
(`tokio-postgres`) and OpenSearch (`reqwest`) against one byte-identical corpus
and **fails the build** if lumen loses any *gated* search cell. The authoritative
thresholds live in **`tests/perf-baseline.json`**; full methodology, per-tier
numbers, resource columns, and reproduction live in
**[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md)**.

The product target is **not** "win the tiny loopback request." Lumen is built for
large index state and sustained request volume over HTTP/2 multiplexed
connections. Low-QPS rows remain in the matrix because they catch regressions
early and explain fixed overhead, but the release-relevant performance claim is
high-QPS / large-corpus stability: throughput, p99, RSS, footprint, and peer
comparison under enough concurrency for HTTP/2 pooling to matter.

How the comparison stays honest (separate metrics, never conflated):

- **End-to-end, single-client** is a smoke/regression metric — lumen and
  OpenSearch share HTTP/JSON so the transport tax is visible. pg's binary wire
  beats HTTP/JSON on cheap btree point/range lookups on loopback, so those cells
  are **HTTP-EXEMPT** (annotated) and gated instead through a **native
  prepared-binary** path (Rust wire over Unix socket) — the cheap predicates
  still carry a hard floor.
- **Concurrent qps (10/100/1000)** and **write-path qps** are report-only by
  default; `LUMEN_GATE_COMPARE_PEERS=1 LUMEN_PERF_STRICT=1` strict-gates the peer
  rows recorded in `perf-baseline.json`. Co-located CI keeps them report-only
  until CPU isolation; isolated-host high-QPS repeats are the release-stable bar.

Each cell carries a threshold in `perf-baseline.json`: a **WIN cell** must hold
`max(1.0, 0.8 × recorded margin)` — a **ratchet**, so improving a cell locks the
new bar and it can only get better. **HTTP-EXEMPT cells** (pg btree lookups on
loopback) are separately gated by `pg_native` floors through the native path.
**Scale tiers:** 1K smoke/trend, **10K routine AW/release regression**,
**100K explicit release-local calibration**, and 1M release-soak/research only.
The historical 1M proof is retained evidence; refresh it only with an explicit
soak (`LUMEN_GATE_RELEASE_SOAK=1` or `LUMEN_GATE_N=1000000`).

**Current status — GREEN** (routine gate defaults to 10K Lumen-only regression;
retained historical N=1M in-memory + disk-tier peer evidence). Representative
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

**Write path** — `tests/write_qps.rs` drives the real HTTP `POST /index`; the
legacy NATS/JetStream row remains the historical write-path comparison while
the serving/operator HA path uses Lumen-owned raft. Latest historical 100-worker JetStream run: **8.5× vs
Postgres**, **3.4× vs OpenSearch**, 0 errors. `LUMEN_PERF_STRICT=1` strict-gates
the write margins only when peer services are explicitly present; per-mode
numbers and tuning history live in `benchmarks-scale.md`.

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

Full row-count x qps scaling, footprint tables, and retained vs-pg / vs-OS
breakdowns live in **[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md)**.
Routine checks use the Lumen-only vat runner; peer comparisons are refreshed
only through explicit calibration/soak runners when a benchmark cell or peer
configuration changes. Docs-per-shard sizing and the per-shard
indexing/search throughput envelope derived from these same bench surfaces
live under "Capacity guidance" in the Elastic Scale capability above.

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
| `hash`    | Caller-supplied 64-bit perceptual/structural hash stored as hex bits         | `hamming { hash, max_distance }` | No; use `hamming` for near-duplicate lookup |

Analyzers available for `text`: `jieba` (Chinese), `whitespace_lower`
(English / generic), `ngram` (configurable min/max). A field is bound
to one analyzer at declaration time.

A field cannot be both `text` and `keyword`. If both are needed (e.g.
"search by email substring *and* find duplicate emails"), declare two
fields and write twice — this keeps write amplification predictable.

## Search concept boundaries

The parity promise is search-side breadth over Lumen's declared contract, not
an implicit claim that every PostGIS/OpenSearch/MongoDB search feature already
exists. These concepts are explicit so agents can choose the right engine or
adapter boundary:

| Concept | Disposition |
|---------|-------------|
| Geo / spatial search | **Roadmap candidate.** Use PostGIS/MongoDB/OpenSearch or a caller-owned geospatial prefilter today, then pass matching `external_id`s to lumen. |
| Phrase / proximity queries | **Roadmap candidate.** Current `match` is bag-of-words BM25 over analyzer tokens, not phrase order or slop. |
| Fuzzy / typo tolerance | **Roadmap candidate.** No edit-distance automaton today; for coarse prefix/substring recall, use the `ngram` analyzer recipe. |
| Synonyms | **Caller-owned.** Expand queries before calling lumen or write normalized companion fields; there is no managed synonym dictionary/analyzer. |
| Autocomplete / suggest | **Recipe.** Declare a dedicated `text` field with `analyzer: "ngram"` and run `match`; lumen returns candidate `external_id`s, not suggestion payloads. |
| Highlighting | **Non-goal.** Search responses contain only `external_id` + `score`; lumen does not store source text to return snippets/fragments. |
| Per-field / per-clause boost | **Boundary.** No arbitrary boost knob today; use separate fields/query legs plus `rrf`, then rerank in the caller if needed. |
| Document TTL / expiry | **Caller-owned lifecycle.** Delete/reindex expired `external_id`s from the source-of-truth event stream; collection soft-delete grace is not per-document TTL. |

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
                   "backend": "hnsw-cpu", "quantize": "sq" },
    "avatar_phash": { "type": "hash" }
  }
}
→ 200 { "collection_id": "users", "version": 1, "fields_count": 6 }
```

Online: adding a new field is immediate (postings start empty).
Re-declaring an existing field with the same spec is a no-op (PUT is
upsert-merge). Changing a field's type is rejected — drop the field
(`DELETE /collections/{id}/fields/{name}`) and re-add. `vector` field
configuration (`dim` / `metric` / `backend` / `quantize`) is immutable
for the field's lifetime. `hash` has no schema-time hash-kind parameter:
the caller computes pHash, SimHash, b-bit MinHash, or another 64-bit signature
and writes it as a 16-hex-character string (optional `0x` prefix accepted).

### Index (write)

```
POST /collections/{id}/index
{
  "items": [
    { "external_id": "u_123", "field": "bio",   "value": "senior engineer in Taipei" },
    { "external_id": "u_123", "field": "email", "value": "a@x.com" },
    { "external_id": "u_123", "field": "tags",  "value": ["rust","db"] },
    { "external_id": "u_123", "field": "avatar_phash", "value": "f0e1d2c3b4a59687" }
  ],
  "request_id": "..."        // optional, dedup TTL 5 min
}
→ 200 { "indexed": 4, "bytes_written": { "bio": 412, "email": 33, "tags": 88, "avatar_phash": 12 }, "shard_lag_ms": 4 }
```

Re-writing `(external_id, field)` fully re-indexes that field. There
is no partial update. `/index` is a **merge**: only the fields you send are
touched. Own only some fields of a doc? Use `/index`. Own the doc's
**complete** row? Use `docs:replace` below.

### Full-replacement writes (docs:replace)

```
PUT /collections/{id}/docs:replace
{ "docs": [
    { "external_id": "row-42", "version": 7, "fields": { "title": "New title", "state": "open" } }
] }
→ 200 { "results": [
    { "status": "ok", "fields_written": 2, "fields_skipped": 0 }
] }

PUT /collections/{id}/docs/{external_id}          # single-resource sugar
{ "version": 7, "fields": { "title": "New title", "state": "open" } }
→ 200 { "status": "ok", "fields_written": 2, "fields_skipped": 0 }
```

`docs:replace` is a batch **full-replacement** upsert: each item's `fields`
becomes the doc's *entire* indexed state — a declared schema field the doc
has today but that is absent from `fields` is **implicitly deleted**.
`docs:replace` is one literal path segment appended after
`{collection_id}` (AIP-136 custom-method syntax), so it registers directly
in axum next to `/collections/{collection_id}/docs/{external_id}` with no
capture ambiguity — collection ids may not contain `:` for the same reason.

**PUT is deliberate**: this is idempotent full replacement, so replaying
the same request converges to the same state. **Own the complete row for a
doc? Use `docs:replace`. Own only some fields? Use `/index`** — `/index` is
a merge, `docs:replace` is a full replacement.

`version` is optional **doc-level** last-write-wins over the caller's own
source-row version — distinct from `/index`'s `IndexItem.version`, which is
per-`(external_id, field)` cell versioning. A strictly-older version
arriving later drops the *entire* item and is reported as its own
`{"status":"dropped","current_version":...}` result, kept separate from
both `ok` and `error` so callers can tell "a newer write already won" apart
from both success and failure. Each `ok` result carries `fields_written`
and `fields_skipped` counters; `fields_skipped` (unchanged-value no-op
suppression) is always `0` today.

**A per-item failure never fails the batch**: the batch-level HTTP status
stays 200 unless the body is malformed or the batch is over the size limit
(max 32 items — `MAX_BATCH_REPLACE_SIZE`, the same knob family as
`collections:search`'s `MAX_BATCH_SEARCH_SIZE`) — those return 400.
`PUT /collections/{id}/docs/{external_id}` is single-resource sugar for a
one-item batch, unwrapped back into a bare per-item result.

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

**`range` also accepts a string bound** (`gte`/`lte`/`gt`/`lt`) against a
`keyword` field for byte-lexicographic (string/date) ranges, e.g.
`{ "range": { "field": "created_at", "gte": "2026-01-01", "lt": "2026-02-01" } }`
— a numeric bound against a `keyword` field or a string bound against a
`number` field is rejected with 400 rather than silently misparsed.

**`QUERY /collections/{id}` is a dual-registered twin of this endpoint**
(RFC 10008): same request body, same handler, byte-identical response.
`OPTIONS`/`HEAD` on either target advertise `Accept-Query: application/json`.
POST remains the permanent fallback for clients without QUERY support.

The **`X-Read-Consistency`** request header (`leader` / `bounded(<ms>)` / `any`
— default and safest is `leader`; missing/unrecognized values also fall back
to `leader`, an owner decision kept as-is with no formal release yet to
force a compatibility bar) is enforced against live cluster state in
primary-replica (raft) mode: `leader` only succeeds on the pod currently
holding leadership, and a request that fails the check is rejected rather
than silently served from a possibly-stale replica. **`bounded(<ms>)`
succeeds on the leader (never stale) but currently always rejects on a
follower/learner**: lumen does not yet measure real replication lag between
peers, so a non-leader replica reports the conservative "lag unknown"
sentinel and is treated as over any bound rather than risk serving a stale
read. Real follower lag reporting (and `bounded` actually succeeding on a
caught-up follower) is future work — until then, `bounded(<ms>)` is
effectively `leader` with an extra rejection path for followers. Standalone
deployments (no raft) ignore the header.

### Batch search (msearch-style, multi-collection)

```
POST /collections:search
{ "searches": [
    { "collection": "users",    "query": { "term": { "field": "tags", "value": "rust" } }, "limit": 10 },
    { "collection": "products", "query": { "match": { "field": "title", "text": "earbuds" } }, "limit": 5 }
] }
→ 200 { "results": [
    { "status": "ok", "response": { "hits": [...], "total": 2, "cursor": null, "took_ms": 1 } },
    { "status": "error", "code": "collection_not_found", "message": "..." }
] }
```

`collections:search` is one literal path segment (AIP-136 custom-method
syntax), so it registers alongside `/collections/{collection_id}` with no
ambiguity — for the same reason, collection ids may not contain `:`. Each
item is a full `{"collection", ...SearchRequest}` — `limit`, `sort`,
`cursor`, `collapse`, `routing_key`, `track_total` may all differ per item.
`results` has the same order and length as `searches`. **A per-item failure
never fails the batch**: the batch-level HTTP status stays 200 unless the
body is malformed or the batch is over the size limit (max 32 items, which
also bounds the concurrent fan-out) — those return 400. Pagination stays
per-item: resubmit one item with its returned `cursor` to continue it. There
is no merged cursor and no cross-collection score merging/ranking.

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
POST /admin/backup:scoped                         # SnapshotV1 restricted to a set of virtual buckets
POST /admin/reshard:apply                         # additively merge one ReshardBatch into live state
POST /admin/reshard:evict                         # remove docs no longer owned under a newer shard map
POST /admin/reshard:fence                         # arm/clear a bounded write pause on a set of buckets
POST /admin/reshard:prune                         # accumulate + prune the final migration pass's keep set
POST /admin/checkpoint                            # force a synchronous full-state durability checkpoint
GET  /debug/cluster                               # pod/shard/role/peers/replication-lag
GET  /metrics                                     # Prometheus text format
GET  /healthz                                     # liveness
GET  /readyz                                      # readiness (503 while draining)
GET  /openapi.json                                # live OpenAPI spec
GET  /docs                                        # Swagger UI (interactive "Try it out")
```

`backup:scoped` / `reshard:apply` / `reshard:evict` / `reshard:fence` /
`reshard:prune` are the operator-driven reshard data-plane verbs:
`backup:scoped` exports only the documents routed to a requested set of
virtual buckets (the same hash the engine's own routing uses),
`reshard:apply` idempotently upserts one such export's batch into a target
shard, `reshard:evict` removes exactly the documents a supplied newer
virtual-bucket map no longer routes to that shard, `reshard:fence` arms or
clears a bounded (default 300s, max 3600s) write pause on a set of buckets
so a write mid-cutover is rejected with a retryable `503
bucket_write_paused` instead of racing the map change, and `reshard:prune`
accumulates a final migration pass's authoritative per-bucket "keep" id set
and prunes anything absent from it once complete. All five are
`Role::Admin`-gated and idempotent on retry; `reshard:fence` is
driver-owned (`service_k8s::reshard_driver::advance_catching_up`) — manual use
outside driver-orchestrated cutover risks a real write outage.
`reshard:apply`/`reshard:evict` mutate engine state directly, bypassing the
normal WriteCoordinator/AOF write path, so the reshard driver calls `POST
/admin/checkpoint` — a synchronous full-state segment checkpoint returning
`{"persisted": bool}` — on every touched shard before cutover, making the
migration durable ahead of the rolling-restart that flips the live shard
map.

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
streams, which is how lumen sustains its high-QPS search/index throughput. Small
HTTP/1.1 calls are compatibility and smoke paths; production performance claims
are about pooled HTTP/2 traffic at volume. The
three setups, in order of preference:

- **Production (private ClusterIP TLS) — HTTP/2 by default, for free.** Lumen
  terminates TLS itself on `https://<instance>.<namespace>.svc:7373` and offers
  ALPN `h2, http/1.1`, so every client gets h2 transparently. Nothing sits in
  front: no ingress, no mesh, no other TLS terminator (see
  [Authentication and authorization](#authentication-and-authorization)). This
  is the recommended deployment.
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

### Authentication and authorization

> This section describes what the binary enforces, not a target. The bearer
> registry, Google-token verifier, Secret Manager/CSI auth projection,
> metadata-server token path, and token environment injection are removed —
> the CRD fields that configured them no longer exist, so a manifest that
> restores one is rejected by the API server rather than quietly ignored.

#### Externally Provisioned TLS Secrets

Deployment administrators or an external platform provision the serving and peer
TLS Secrets named by each `Lumen` instance. The operator only consumes those
Secrets; it does not resolve issuers, perform CAS automation, or own a trust
domain.

Two independent checks stand between a caller and a collection, and neither
substitutes for the other: the **transport** proves which server you reached,
and the **request identity** proves who is asking.

#### Transport: private ClusterIP TLS, terminated by lumen

Production traffic is **not** published. An instance is reached at its Service
DNS name inside the cluster and nowhere else:

```text
LUMEN_URL=https://<instance>.<namespace>.svc:7373
```

There is no Ingress, no Gateway, no LoadBalancer, no NodePort, and no service
mesh terminating TLS on lumen's behalf. The serving pod holds the private key
itself, so the connection a caller authenticates is the connection lumen
serves — an edge that terminated TLS and re-originated plaintext would carry
the KSA token over an unauthenticated last hop while every client-side check
still passed.

`spec.servingTlsSecret` names the Secret holding `tls.crt`, `tls.key`, and
`ca.crt`; the operator projects it into every serving pod and switches the
client port from h2c to TLS with ALPN `h2, http/1.1`. The leaf asserts the
Service's own two DNS spellings and nothing else — a name in the certificate is
a name the instance can impersonate. While no valid leaf is active the port
refuses connections rather than falling back to plaintext. Omit the field only
for local and kind development.

Callers verify against the anchor alone: the deployment administrator or an
external certificate platform distributes the public CA separately from the
private-key-bearing serving Secret. Supply that CA with `lumen connect
--ca-file`, or as `PrivateTrust` in a generated client. It replaces the public
roots rather than joining them, so a public CA cannot vouch for this private
Service DNS name.

#### Request identity: a short-lived KSA token the cluster answers for

For `spec.auth: required`, Lumen accepts only a short-lived Kubernetes
ServiceAccount token with audience `lumen.axiom.dev`:

```text
Google user or Google service account
  -> authenticate to kube-apiserver through kubeconfig
  -> RBAC-authorized TokenRequest for one explicitly named client KSA
  -> short-lived KSA token
  -> Lumen TokenReview
  -> system:serviceaccount:<namespace>:<name>
  -> Lumen SubjectAccessReview
```

Google credentials stop at kube-apiserver. A Google access token, Google ID
token, ADC credential, GSA credential, or metadata-server token sent directly
to Lumen is rejected even if GKE would accept that principal at the Kubernetes
API boundary.

Lumen maps authenticated requests to virtual Kubernetes resources in API group
`lumen.axiom.dev`:

| Lumen decision | Kubernetes resource attribute |
|---|---|
| read one collection | `get` on `lumencollections/<collection-id>` |
| write one collection | `update` on `lumencollections/<collection-id>` |
| administer one collection | `delete` on `lumencollections/<collection-id>` |
| instance-level administration | the corresponding verb on `lumenadmin` |

The request namespace is part of every decision. Collection-list and
multi-collection operations authorize the concrete resources they touch; an
instance admin grant is not modeled as wildcard access to every collection.
Authentication failures return 401 and authenticated denials return 403.

The Lumen CLI uses the current kubeconfig, including the GKE credential plugin,
to request a 600-second token for an explicitly supplied namespace and client
KSA. `lumen query` keeps the token in memory. `lumen connect` gives its child
only a loopback URL and injects the header in a local proxy; it does not expose
the token through environment, argv, files, clipboard, or stdout.

The account is named per invocation and never inferred: `--client-sa` has no
environment fallback and the CLI does not pick a ServiceAccount by listing the
namespace. Omit it and the connection carries no identity at all, which is
correct only against a fleet with `auth: disabled`. Minting needs `create` on
that ServiceAccount's `token` subresource; `lumen k8s access render` emits the
grant, and a refusal names the Kubernetes username the cluster saw, the target
account, and the `kubectl auth can-i` that answers "may I?".

Serving, operator/reshard, backup, and external-client ServiceAccounts are
separate identities with least-privilege bindings. TokenRequest permission is
restricted to one named client KSA and is never a namespace-wide wildcard.
Probe/spec/scrape routes (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`,
`/docs`) remain auth-exempt.

Raft peer identity is a separate plane. Replicated traffic on `:7374` requires
an instance-scoped X.509 certificate and mTLS, with no plaintext fallback. A
KSA token does not authenticate a peer, and a peer certificate grants no
collection or admin access. `spec.peerTlsSecret` is a separate field from
`spec.servingTlsSecret` for that reason: sharing one Secret would let either
listener's material authenticate on the other's port.

Public exposure of any shape — Gateway, Ingress, LoadBalancer, NodePort, VPN,
or a mesh terminating TLS — is outside the Security & Access capability, as are
Google IAM automation and general user/group management. So is client mTLS:
a caller proves who it is with a KSA token, not with a certificate.

## OpenAPI

| Artefact              | When to use                                                  |
|-----------------------|--------------------------------------------------------------|
| `GET /openapi.json`   | Live spec from a running pod — codegen against an actual env |
| `GET /docs`           | Interactive Swagger UI ("Try it out")                        |
| `lumen spec`          | Offline OpenAPI JSON from the installed binary               |
| `lumen spec --format openapi-yaml` | Offline OpenAPI YAML for agent review         |
| `lumen spec --format json-schema` | Component schemas for the request/response types |
| `lumen spec gen --lang ts\|py\|rust [--target <profile>] --out <dir>` | In-tree typed client generation with a pinned target contract |

`lumen spec` and the live endpoint generate from the same Rust code
(`#[derive(utoipa::OpenApi)]` on `api::ApiDoc`). There is no committed OpenAPI
snapshot; the binary and live endpoint are the source of truth.

`projects/lumen/clients/codegen.toml` pins the default TypeScript, Python, and
Rust targets. `spec gen` writes `.openapi-codegen.json` beside every
generated client; use `--target python-3.11` (or another supported profile)
only for a deliberate one-off compatibility override.

Generated Python clients include pydantic models plus a stdlib HTTP/2 runtime.
For auth-enabled deployments:

```python
from generated_api import Client

client = Client("http://lumen.default.svc.cluster.local:7373", auth_token="...")
```

`default_headers={"Authorization": "Bearer ..."}` is also supported. The
generated `h2c_runtime.py` exposes unary `request()` and bidirectional
`stream()` APIs; Lumen's current OpenAPI routes are unary, so generated
`client.py` uses `request()` today and the streaming surface is forward-looking
runtime capacity for services that add streaming operations.

## Design notes (from the retired HA.md, 2026-07)

Durable decisions folded from the retired `HA.md`; its session-era "Original
design notes (openraft)" framing was already superseded by the shipped
`raft-core`/`raft-runtime` implementation and is dropped as historical.

lumen is a **log-replicated, derived, rebuildable search index**: the caller
still owns the source of truth, and lumen indexes the caller's `external_id`s.
The deployment boundary changed once `libs/raft-core` existed: multi-pod lumen
owns its own write ordering and replica synchronization instead of requiring
an external broker as the default HA path. Mode split:

- **standalone**: one pod, embedded WAL, direct apply.
- **primary-replica**: multiple lumen pods, `raft-core` elects a leader, the
  leader owns the ordered write log, and followers replicate/apply the same
  raw `WalRecord::encode()` bytes.

`lumen serve --wal auto` is the production default: it starts embedded when no
k8s replica topology is present, and switches to raft when
`REPLICAS_PER_SHARD > 1` is injected by the operator/StatefulSet. The storage
topology contract is `totalPods = shardCount * replicasPerShard`:
`replicasPerShard` selects the HA mode for each shard group, while
`shardCount` selects how many physical storage shards own the corpus. A
deployment with `shardCount > 1` and `replicasPerShard = 1` is sharded but not
raft-replicated; a deployment with `shardCount = 1` and `replicasPerShard > 1`
is one shard with raft replicas. StatefulSet pod ordinals map deterministically
to `shardIndex = ordinal % shardCount` and
`replicaIndex = ordinal / shardCount`.

The operator never passes special cluster flags — topology comes from the
downward API (`POD_NAME`, `POD_NAMESPACE`, `SHARD_COUNT`,
`REPLICAS_PER_SHARD`, `VOTER_COUNT`, `LUMEN_HEADLESS_SERVICE`): one serving pod
renders a standalone Deployment + HPA, `replicasPerShard > 1` renders stable
serving StatefulSets + headless Services. For local multi-node work,
`LUMEN_PEERS=host:port,...` overrides headless DNS so several
`lumen serve --wal raft` processes can run on one machine.

Dynamic shard growth is an operator workflow, not a direct HPA response. The
default routing contract uses virtual buckets:
`bucket = hash(collection_id, routing_key || external_id) % virtualBucketCount`,
then a versioned bucket-to-physical-shard map decides ownership. Search without
a routing key scatters/gathers across shards; search with a routing key can
target one shard. Operators should prepare a split around storage pressure
(for example 50% of the configured shard ceiling), start or recommend split
based on growth and safety windows, treat high utilization as urgent, and avoid
auto-split when the max shard size or max shard count is unknown.

Raft responsibility is split by crate/module: `libs/raft-core` (consensus
state machine and log semantics), `libs/raft-runtime` (h2c peer transport,
leader forwarding, snapshot install, log compaction — snapshot upload/pruning
policy lives in `libs/service-backup`), `apps/lumen/src/raft_sm.rs`
(committed write records → engine mutations, snapshot produce/restore), and
`apps/lumen/src/raft.rs` (API-facing cluster/debug DTOs, read-consistency
parsing). Legacy broker-backed write logs are not part of the Lumen
deployment archetype; the NATS backend is compatibility/test surface only, and
Relay WAL support has been removed from Lumen.

Bootstrap modes are intentionally distinct. A restarted pod with its PVC
replays local raft state, snapshots, and logs. A new empty-PVC replica can catch
up through leader snapshot install and AppendEntries today, but the production
path is to seed from object-store/shard snapshot first and then catch up the
raft delta, with operator-visible progress and rate limits. Production Lumen
CRs must configure scheduled object snapshots; local filesystem snapshots are
local-dev or break-glass only. External backup is the cold disaster-recovery
and seed surface; it is not the normal live replica synchronization path.
