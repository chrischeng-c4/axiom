# Phase-2 Baseline (ctx_inf_db @ 55138b33)

Measured on: 2026-04-16, Apple M1 Pro, macOS 14.6.1 (Darwin 23.6.0).
Run: `cargo bench -p cclab-ctx-inf-db --bench ingest_throughput`.

## Single-thread ingest

| Benchmark              | Throughput (ops/sec) | p50 latency (us) | p99 latency (us) |
|------------------------|---------------------:|-----------------:|-----------------:|
| entity_append          |              915,370 |            1.121 |            4.189 |
| relation_append        |              219,190 |            4.854 |            8.516 |

Notes:
- `entity_append` throughput uses criterion's slope-fit point estimate; the
  raw per-sample mean is ~700 K ops/sec (the slope estimate excludes one-off
  setup latency that linear regression isolates from per-iter cost).
- `relation_append` is ~4 x slower per op than `entity_append`; the additional
  cost is the source / target existence check plus two adjacency-list pushes.

## Multi-thread ingest (4 threads)

| Benchmark              | Aggregate ops/sec    | Per-thread ops/sec | Scaling factor |
|------------------------|---------------------:|-------------------:|---------------:|
| entity_append          |            1,242,700 |            310,675 |       1.36 x   |

Scaling factor is computed against the single-thread `entity_append` slope
estimate (915,370 ops/sec).

## Observations

- Single-thread `create_entity` runs at ~1 us / op; ~70 % of that is the
  `Entity::new` constructor (Uuid::now_v7 + Utc::now syscall + 11 field
  allocation), measured indirectly by comparing to a no-op closure.
- `create_relation` per-op cost is ~4 x `create_entity` because of the two
  endpoint existence probes against `entities` plus two adjacency-list
  pushes; this is in line with the Phase-1 design.
- 4-thread scaling is only ~1.36 x (not the ideal 4 x), indicating contention
  on the bounded WAL command channel (10,000 slots) and/or the DashMap shards
  for `entities` / `adj_out` / `adj_in`. No `ops_dropped_on_full` was directly
  observed because `PersistenceStats` is `pub(crate)` and unreachable from
  `benches/`; criterion's iteration-completed counter matched the expected
  40,000 ops per multi-thread iteration (asserted via
  `engine.stats().entity_count`).
- `BUG-WAL-ROTATE-COLLISION` (cclab-wal `WalWriter::rotate` at writer.rs:127)
  did NOT visibly fire in this run: criterion's iteration counts and the
  engine's `stats().entity_count` matched the expected 40,000 per
  multi-thread iteration, with no panic / deadlock / data loss observed. The
  64 KiB rotation threshold is small enough that the bug is plausible under
  longer runs; this baseline does not contradict the bug, only documents that
  the current 4 t x 10 k workload did not surface it.
- Total bench wall time: 118 s (well under the 10-minute ceiling); see
  `Re-measurement` below for the run command.

## Deviations from spec

- R2b `bench_single_thread_relation_append`: `sample_size` reduced from 100
  (spec) to 20. With criterion's `iter_custom`, the 1 000-entity seed runs
  once per sample at ~1 ms / seed entity (~1 s seed cost per sample); 100
  samples would push this single bench past the 10-minute total runtime
  ceiling. 20 samples still gives a tight CI (8 % half-width).
- R2c `bench_multi_thread_entity_append`: `sample_size` reduced from 30 to
  10, `measurement_time` reduced from 15 s to 10 s — both per the PM-supplied
  fallback in the issue. Criterion's slope-fit estimate is stable at this
  sample count (CI half-width ~ 5 %).

## Re-measurement

To refresh these numbers after a code change, run the bench command above and
overwrite this file. This file is NOT auto-generated; the run script is the
bench command.
