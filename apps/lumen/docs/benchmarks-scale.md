# lumen - scale benchmark results

This page tracks lumen's scale posture across row count, read/search latency,
paced qps, write qps, and disk footprint. The authoritative regression contract
lives in `apps/lumen/e2e/perf-baseline.json`; this document is the human
handoff for what the current numbers mean.

> **Status (2026-06-08 calibration):** the retained competitive calibration was
> run at `N=1,000,000` docs. At that size lumen cleared every OpenSearch
> read/search cell in both in-memory and segment-disk modes, cleared the
> `LUMEN_PERF_STRICT=1` qps100 and qps1000 OpenSearch + pg-native rows with no
> TARGET rows, and cleared the NATS write gate against both Postgres and
> OpenSearch. Routine AW/release checks do not remeasure Postgres/OpenSearch;
> they run a 10K Lumen-only regression against retained calibrated floors.
> Refresh peer data only when benchmark cells, peer versions/configuration, or
> an explicit release-soak request require it.

## Benchmark Standard

| axis | standard |
|---|---|
| Engines | lumen by default; PostgreSQL/OpenSearch only for explicit calibration |
| Data sizes | **1K, 10K, and 100K standard scale matrix**; above 100K is explicit release-soak/research |
| QPS targets | 10, 100, 1000 paced requests/sec |
| Latency metrics | p50, p95, p99 |
| Reliability | error rate / timeout / status / transport failures |
| Resource metrics | CPU, memory/RSS, disk IO |
| Footprint metric | **disk size**: on-disk MiB and bytes/doc |

The standard 100K cap keeps local evidence bounded. The full standard matrix
uses every row size and every QPS target. QPS applies only to reads. Batch
unindex and truncate are one logical request each, never paced load tests.

Disk size belongs in the official matrix. It is different from disk IO: disk IO
captures runtime pressure during a load window, while disk size captures storage
footprint for the same corpus. The competitive gate now reports same-corpus
footprint for lumen (`.lseg` bytes), PostgreSQL (`pg_total_relation_size`), and
OpenSearch (`_stats/store`). The qps table also reports process-level
`RUSAGE_SELF` resource columns for the lumen in-process test server:
`cpu_ms`, `rss_mib`, minor/major page faults, and block input/output counts.
Full server-side CPU/RSS/disk-IO isolation for PostgreSQL and OpenSearch belongs
in a dedicated perf-host report. It is not part of routine AW/release checks.

## How To Reproduce

Fast developer-loop checks should stay out of release perf mode. Use `cargo
check`/focused debug tests for API/routing semantics, and reserve release mode
for perf evidence. A cold debug cache can still take minutes because dependencies
must be checked once; the same target is about a second after the cache is warm:

```sh
cargo check -p lumen --test api_e2e --test write_qps
cargo test -p lumen --test api_e2e index_can_use_injected_sharded_write_backend -- --nocapture
```

Use release compile checks when a bench or optimized-only code path changes:

```sh
cargo test --release -p lumen --test write_qps --no-run
```

lumen-only disk scale bench:

```sh
LUMEN_SCALE_DISK=1 LUMEN_SCALE_QPS=1 LUMEN_SCALE_CELLS=range,filter_sort,keyword_sort,sorted_page_deep LUMEN_SCALE_QPS_TARGETS=10 apps/lumen/scripts/lumen_scale.sh 1000

LUMEN_SCALE_DISK=1 LUMEN_SCALE_QPS=1 LUMEN_SCALE_CELLS=range,filter_sort,keyword_sort,sorted_page_deep LUMEN_SCALE_QPS_TARGETS=10,100,1000 apps/lumen/scripts/lumen_scale.sh 1000,10000,100000

LUMEN_SCALE_ALLOW_ABOVE_STANDARD=1 LUMEN_GATE_WINDOW_S=0.2 LUMEN_SCALE_CHUNK_ROWS=100000 ./scripts/lumen_scale.sh 1000000
```

The first command is the DEV command. It runs only `N=1,000` and QPS `10`.
The second command is the full command. It runs `N=1,000`, `10,000`, and
`100,000` with QPS `10`, `100`, and `1,000` for every fixed read cell.

The bench stream-generates docs, so it no longer allocates a full corpus Vec.
The single-segment path still builds the mutable Engine index in RAM before
`flush_to_segments`; the standard script and test harness default to
`1,000 / 10,000 / 100,000` rows and reject larger row counts unless an explicit
release-soak/research run sets `LUMEN_SCALE_ALLOW_ABOVE_STANDARD=1`. The point is
operational: routine readiness should stay fast, while retained 1M probes remain
available when scale evidence is actually needed.

The four fixed selectors are `range`, `filter_sort`, `keyword_sort`, and
`sorted_page_deep`. `range` combines range and boolean filtering. `filter_sort`
uses an ascending number sort. `keyword_sort` uses a deterministic
high-cardinality keyword sort with sparse values, `missing:last`,
`track_total:true`, and limit 80. Nine out of ten rows have a unique sort
value. `sorted_page_deep` uses cursor pagination.
Before it measures a normal single-engine row, the bench uses HTTP to prove
`/stats.documents_indexed == N`. It checks fixture-document order for `range`.
It checks age-then-fixture-document order for `filter_sort`, the first 80
present sort-key rows for `keyword_sort`, and `d0` to `d99` for the first `sorted_page_deep`
page. Before the cursor QPS cell, the bench walks to a precomputed mid-collection
cursor and proves two ordered, non-overlapping pages. It does
not keep issuing page-one queries. Each report row contains documents, selector,
target QPS, achieved QPS, p50/p95/p99, errors, error rate, and `ok`, `SVR`, or
`HARN`. Any HTTP request error fails the test after the row is reported. These
status values classify the observed target and `/healthz` ceiling. They are not
latency or achieved-QPS thresholds. Before this warm matrix, `keyword_sort`
also reports a response-cache-cold request for limits 1, 20, 80, 500, and
2,000. After the batch reindex and temporary-survivor cleanup, it reports one
more cache-invalidated cold request at limit 80. That request checks the exact
page and total after sealed postings merge with the live write tail. These rows
report timing only; they have no latency threshold.

After all reads at each `N`, the normal single-engine HTTP path sends one atomic
batch unindex for exactly `min(1000,N)` deterministic IDs. It checks that all
removed IDs are absent and that an added temporary survivor is still present.
The survivor is necessary at `N=1,000`, where the required batch removes every
measured document. The bench reindexes the selected IDs, removes the temporary
survivor, and then measures truncate with exactly `N` documents again. It
records the two logical request latencies and immediate `/stats` visibility.
For truncate, it takes read-only reclaimer snapshots before and after the
request. The report includes pending generations, queued and active tasks,
submitted and completed generations, the actual shared-queue high-water mark,
and drain time from the truncate HTTP request start. The harness waits at most
30 seconds for this truncate generation to complete and for pending generations
to return to the baseline. A timeout fails the harness. The HTTP truncate
acknowledgement remains a separate logical result. Chunked research mode has
only a read router, so it reports the mutation probe as unavailable rather than
claiming cross-shard atomicity.

Chunked reopened mode remains useful for explicit 1M soaks. The smoke command above
builds ten 100k-doc sealed chunks, reopens them as read shards, and runs the qps
ladder through the real API search backend seam:
`/collections/docs/search` dispatches to an injected `SearchBackend`, fan-ins
with `lumen::routing::search_shards_parallel`, and serializes the normal HTTP
JSON response. The historical 1M / 2 chunk smoke had qps1000 p50
**0.201-1.358 ms** across its former cells with **27.62 MiB** on disk. Its
targeted `kw_term` selector is retired and is not a result for the fixed 0.4.31
scale matrix. Use `LUMEN_SCALE_CELLS=range` for a current focused research
selector. `LUMEN_SCALE_CHUNK_WORKERS=N` parallelizes
chunk-level build/write/seal/reopen for these bounded 1M probes. This is
chunk-level parallelism with independent Engines, not a multi-threaded write
lock inside one collection.

Routine Lumen-only read/search regression gate:

```sh
cargo test --release -p lumen --test perf_gate_vs_db competitive_perf_gate -- --ignored --nocapture
```

Explicit peer calibration gate against Postgres + OpenSearch:

```sh
LUMEN_GATE_COMPARE_PEERS=1 LUMEN_GATE_N=100000 cargo test --release -p lumen --test perf_gate_vs_db competitive_perf_gate -- --ignored --exact --nocapture
```

Explicit strict qps proof for qps10/qps100/qps1000 OpenSearch and pg-native rows:

```sh
LUMEN_GATE_COMPARE_PEERS=1 LUMEN_PERF_STRICT=1 cargo test --release -p lumen --test perf_gate_vs_db competitive_perf_gate -- --ignored --exact --nocapture
LUMEN_GATE_COMPARE_PEERS=1 LUMEN_PERF_STRICT=1 LUMEN_GATE_QPS_TARGETS=10 cargo test --release -p lumen --test perf_gate_vs_db competitive_perf_gate -- --ignored --exact --nocapture
LUMEN_GATE_COMPARE_PEERS=1 LUMEN_PERF_STRICT=1 LUMEN_GATE_CELLS=text_bm25 LUMEN_GATE_QPS_TARGETS=1000 cargo test --release -p lumen --test perf_gate_vs_db competitive_perf_gate -- --ignored --exact --nocapture
```

The second command makes qps10 part of the strict peer diagnostic proof. The
third command is a focused diagnostic path for one qps cell. Defaults keep the
routine runner Lumen-only.

Historical write-path qps gate through the legacy NATS JetStream WAL path:

```sh
LUMEN_WRITE_MODES=embedded,sharded LUMEN_WRITE_WARMUP_S=0.1 LUMEN_WRITE_WINDOW_S=0.3 cargo test --release -p lumen --test write_qps write_qps_bench -- --ignored --nocapture
LUMEN_WRITE_MODES=nats,natssharded LUMEN_WRITE_WARMUP_S=0.1 LUMEN_WRITE_WINDOW_S=1.0 cargo test --release -p lumen --test write_qps write_qps_bench -- --ignored --nocapture
LUMEN_PERF_STRICT=1 cargo test --release -p lumen --test write_qps write_qps_bench -- --ignored --nocapture
```

`tests/write_qps.rs` also reports a local `sharded` lumen leg. It uses
`LUMEN_WRITE_SHARDS` (default 4) to route one HTTP `/index` request by
`external_id` across independent `WriteCoordinator`/`Engine` shards. That row is
for multi-core write-apply exploration. The retained historical JetStream
comparison is against Postgres and OpenSearch; the current serving/operator HA
path uses Lumen-owned raft. The strict historical cell is the single-stream row
(`nats_index_100`); the partitioned row
(`natssharded_index_100`) is a TARGET/trend row until it is stable under the same
timeout/error envelope. `LUMEN_WRITE_MODES=embedded,sharded` plus a short
`LUMEN_WRITE_WARMUP_S`/`LUMEN_WRITE_WINDOW_S` is the fast local trend check; do
not treat it as a replacement for the full strict gate.
`LUMEN_WRITE_MODES=nats,natssharded` compares the historical single JetStream
stream (`lumen_wal` / `lumen.wal`) with the partitioned shape
(`lumen_wal_N` / `lumen.wal.N`) enabled by `NatsWalConfig`. It requires
JetStream (`nats-server -js`), not a plain NATS broker.

Latest warm-cache quick trend (`LUMEN_WRITE_MODES=embedded,sharded`,
`LUMEN_WRITE_WARMUP_S=0.1`, `LUMEN_WRITE_WINDOW_S=1.0`,
`LUMEN_WRITE_BATCH_DOCS=100`, 2026-06-07):

| path | workers=10 docs/s | workers=100 docs/s | p50 at workers=100 |
|---|---:|---:|---:|
| embedded local WAL | 850.1k | 932.5k | 9.623 ms |
| sharded local write backend, 4 shards | 1,258.1k | 1,362.9k | 7.016 ms |

That is a **1.46x** 100-worker improvement over the single-engine embedded path
on the same 1s quick window. The result is useful for direction; the retained
NATS-vs-peer run below is historical benchmark evidence, not the current broker
deployment path.

Retired Relay write-path quick trend (`LUMEN_WRITE_MODES=embedded,relay`,
`LUMEN_WRITE_WARMUP_S=0.1`, `LUMEN_WRITE_WINDOW_S=1.0`,
`LUMEN_WRITE_BATCH_DOCS=100`, `--features relay-wal`, 2026-06-21):

| path | workers=10 docs/s | workers=100 docs/s | p50 at workers=100 | p95 at workers=100 | p99 at workers=100 | errors |
|---|---:|---:|---:|---:|---:|---:|
| embedded local WAL | 927.4k | 941.9k | 9.383 ms | 13.499 ms | 62.442 ms | 0 |
| RelayWal + in-process relay broker | 702.7k | 829.7k | 10.972 ms | 16.246 ms | 39.853 ms | 0 |

Relay was **0.88x** of embedded throughput at 100 workers in this short local
run. That row includes the HTTP/2 broker hop plus the Lumen apply path waiting
for subscriber delivery. This is retained only as historical evidence; Relay WAL
is no longer an active Lumen backend or perf gate.

Latest JetStream trend (`LUMEN_TEST_NATS_URL=nats://localhost:4223`,
`LUMEN_WRITE_MODES=nats,natssharded`, `LUMEN_WRITE_WARMUP_S=0.1`,
`LUMEN_WRITE_WINDOW_S=1.0`, `LUMEN_WRITE_BATCH_DOCS=100`, 2026-06-07):

| path | workers=10 docs/s | workers=100 docs/s | p50 at workers=100 | p99 at workers=100 |
|---|---:|---:|---:|---:|
| single JetStream WAL | 434.5k | 891.7k | 10.167 ms | 56.998 ms |
| partitioned JetStream WAL, 4 streams | 349.5k | 987.0k | 9.870 ms | 21.842 ms |

The partitioned JetStream row is **1.11x** faster at 100 workers and materially
reduces p99 in this 1s quick run. It is useful production-shape evidence, but the
official competitive row remains the single-stream JetStream gate until the
partitioned path is timeout-free at the full 5s strict window.

Latest strict write gate (`LUMEN_PERF_STRICT=1`,
`LUMEN_TEST_NATS_URL=nats://localhost:4223`, 5s window, 2s warmup,
`LUMEN_WRITE_BATCH_DOCS=100`, 2026-06-07):

| path | workers=100 docs/s | p50 at workers=100 | p95 at workers=100 | p99 at workers=100 | errors |
|---|---:|---:|---:|---:|---:|
| single JetStream WAL | 822.6k | 11.166 ms | 14.415 ms | 16.621 ms | 0 |
| Postgres bulk insert + indexes | 96.7k | 97.084 ms | 166.037 ms | 215.448 ms | 0 |
| OpenSearch `_bulk` | 241.9k | 35.533 ms | 71.033 ms | 130.768 ms | 0 |

Strict gate verdicts: single JetStream **8.51x vs pg** and **3.40x vs
OpenSearch**. The same-code full partitioned JetStream trend run timed out at
100 workers (200 timeouts), so `natssharded_index_100` remains TARGET/report-only
instead of a release blocker.

The explicit peer gates need Postgres `dbname=lumenbench` and OpenSearch on
`localhost:9200`. The write peer gate also needs a JetStream broker; set
`LUMEN_TEST_NATS_URL` when it is not on the default local URL.

Corpus per doc: `bio` (Text), `city` (Keyword), and `age` (Number). No vectors in
the competitive search gate.

## Scale Vocabulary

| class | meaning | current repo posture |
|---|---|---|
| Small | thousands of docs | routine AW/release regression and API correctness |
| Medium | 100K to 1M docs | explicit peer calibration or release-soak evidence |
| Above 1M | research-only | not a local benchmark target; use storage-model math and distributed/sharded design work, not developer-machine runs |

## Footprint

lumen's on-disk index is linear in N and converges to about **28.8 bytes/doc** on
the current corpus.

| rows | on-disk index | bytes/doc | note |
|---:|---:|---:|---|
| 1 K | 0.07 MiB | 75.6 | fixed overhead dominates |
| 100 K | 2.80 MiB | 29.4 | measured |
| 1 M | 27.50 MiB | 28.8 | measured |

Same-corpus peer footprint from the latest full gate:

| engine | on-disk bytes | bytes/doc |
|---|---:|---:|
| lumen | 27.50 MiB | 28.8 |
| PostgreSQL | 313.59 MiB | 328.8 |
| OpenSearch | 145.29 MiB | 152.3 |

At the measured 1M footprint, larger corpora can be estimated from roughly
28.8 bytes/doc for lumen's index bytes before peer-engine overhead. Those rows
are projections, not local benchmark targets.

Measured per-field 1M footprint:

| field | type | bytes/doc | note |
|---|---|---:|---|
| `bio` | Text | 15.66 | token dict + compressed postings |
| `age` | Number | 7.77 | raw `f64` forward + sorted-value index + postings |
| `city` | Keyword | 3.95 | raw `u32` dict-id forward + dict + postings |
| `_collection` | external id | 0.12 | sequential external-id column compresses near-free |

The largest remaining footprint lever is bit-packing low-cardinality forward
columns (`age`, `city`), but it is not required for the current competitive gate.

## Current 1M Read/Search Gate

Ratio is `peer_e2e / lumen_e2e`, so `>1` means lumen is faster. Every
OpenSearch row below is now a hard WIN cell in `perf-baseline.json`; the failure
floor is `max(1.0, 0.8 * baseline)`.

| cell | vs OpenSearch, in-memory | vs OpenSearch, segment disk | vs Postgres proof |
|---|---:|---:|---:|
| `text_bm25` | 4.50x | 22.98x | 814.99x HTTP WIN |
| `text_and` | 7.72x | 10.92x | 96.92x HTTP WIN |
| `filtered_search` | 7.26x | 4.64x | 61.41x HTTP WIN |
| `kw_term` | 3.95x | 9.33x | 6.16x native WIN |
| `range` | 5.20x | 11.25x | 2.92x native WIN |
| `bool_filter` | 5.20x | 6.61x | 39.57x native WIN |
| `filter_sort` | 4.08x | 6.02x | 43.94x HTTP WIN |
| `pure_sort` | 3.88x | 5.20x | 83.62x HTTP WIN |

Postgres cheap btree point/range/bool predicates are not judged through lumen's
public HTTP/JSON path because pg's binary protocol is its home turf on loopback.
Those same cells are hard-gated through lumen's prepared compact native wire over
Unix socket/TCP fallback, shown above as `native WIN`.

## Current QPS Gate

The qps ladder is paced fixed-rate load. `qps1000` means 1000 requests/second,
not a 1000-concurrent-worker storm. The qps harness pre-serializes request JSON
and runs lumen HTTP/native servers on dedicated Tokio runtime threads, so
p50/p95/p99 do not include load-client JSON encoding or same-runtime server
contention. The report also includes in-window error rate.
The default run reports qps rows and gates only Lumen qps health. Setting
`LUMEN_GATE_COMPARE_PEERS=1 LUMEN_PERF_STRICT=1` makes peer rows recorded in
`tests/perf-baseline.json` strict (`LUMEN_QPS_GATE=1` still works for qps-only
debug when peer comparison is also enabled, and `LUMEN_GATE_QPS_TARGETS=10`
focuses the low-QPS diagnostic). Harness-bound rows that still beat the peer but
miss the ratcheted margin are retried once after a short cooldown; true losses
still fail. Latest retained strict runs passed with no WIN-cell regressions and
no TARGET rows. The retained retries were qps100 `text_bm25` (2.34x -> 2.63x),
qps10 `range` (2.17x -> 13.12x), and qps10 `bool_filter` (2.36x -> 3.80x) vs
OpenSearch. Repeated top-k serving queries reuse cached results, while
writes/deletes/schema changes clear the cache before mutating postings.

| cell | qps10 vs OpenSearch | qps100 vs OpenSearch | qps1000 vs OpenSearch | qps1000 lumen p50 | qps1000 `/healthz` p50 |
|---|---:|---:|---:|---:|---:|
| `text_bm25` | 9.72x | 2.63x | 5.72x | 0.144 ms | 0.157 ms |
| `text_and` | 8.71x | 4.03x | 5.78x | 0.215 ms | 0.157 ms |
| `filtered_search` | 2.66x | 4.67x | 6.10x | 0.232 ms | 0.157 ms |
| `kw_term` | 13.20x | 3.36x | 2.62x | 0.353 ms | 0.157 ms |
| `range` | 13.12x after retry | 3.85x | 4.40x | 0.281 ms | 0.157 ms |
| `bool_filter` | 3.80x after retry | 3.61x | 4.12x | 0.301 ms | 0.157 ms |
| `filter_sort` | 3.46x | 3.61x | 3.51x | 0.332 ms | 0.157 ms |
| `pure_sort` | 3.33x | 3.20x | 3.18x | 0.324 ms | 0.157 ms |

Postgres cheap predicates are strict-gated through lumen's native prepared wire
against pg prepared Unix-domain-socket statements:

| cell | qps10 vs pg native | qps100 vs pg native | qps1000 vs pg native |
|---|---:|---:|---:|
| `kw_term` | 9.51x | 10.38x | 8.49x |
| `range` | 6.53x | 7.66x | 6.50x |
| `bool_filter` | 42.59x | 41.57x | 29.38x |

The qps rows stay perf-strict-only by default because a single co-located developer box
runs the load client, lumen server, Postgres backends, and OpenSearch JVM
together. The strict gate is useful for local proof; release-stable qps
claims should be repeated on an isolated perf host.

## Current Write Gate

The write comparison uses the real serving path:

- lumen: HTTP `POST /collections/{id}/index` through NATS JetStream WAL, local
  apply, and the storage engine.
- Postgres: batched inserts into a table with text, keyword, number, generated
  tsvector, GIN, and btree indexes.
- OpenSearch: `_bulk` into a single-shard, no-replica index with refresh disabled.

Latest retained strict 5s release run:

| path | workers=100 docs/s |
|---|---:|
| lumen NATS WAL `POST /index` | 822.6k |
| Postgres insert | 96.7k |
| OpenSearch bulk | 241.9k |

That is **8.51x vs Postgres** and **3.40x vs OpenSearch** at the 100-worker
NATS write row. `LUMEN_PERF_STRICT=1` enforces the ratcheted margins from
`perf-baseline.json` only on explicit write-peer runs where NATS, Postgres, and
OpenSearch are all reachable; `LUMEN_WRITE_GATE=1` remains available for
write-only debug.

Direct `Engine::index` isolates the storage hot path from HTTP/WAL and currently
measures about **912.5k docs/s** at 100 workers on the same 100-doc batch shape.

## Known Remaining Work

| work | why it remains |
|---|---|
| isolated qps/write perf host | Converts co-located perf-strict gates into release-stable throughput claims. |
| forward-column bit packing | Footprint optimization, not required for the 1M competitive gate. |
| vector disk/GPU backends | Outside the current no-vector competitive gate. |
