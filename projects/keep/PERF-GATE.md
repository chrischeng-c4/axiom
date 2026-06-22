# keep perf-gate (#126)

Two complementary gates, both driven by `meter` (the workspace's resource
measurement tool). One guards **engine throughput** (regressions in the store
itself), the other guards the **server's resource footprint** (memory bloat /
gross CPU regressions). Neither needs a competitor process.

## 1. Engine throughput ratchet — `benches/engine_bench.rs`

Deterministic, network-free criterion microbenchmarks of the sharded engine.
Run them through meter (which delegates `cargo bench` and, with a baseline,
folds regressions to a non-zero exit):

```bash
meter bench --target projects/keep --human          # run the gate benchmark
# or directly:
cargo bench -p keep --bench engine_bench
```

Record / compare a baseline (criterion):

```bash
cargo bench -p keep --bench engine_bench -- --save-baseline keep-gate   # record
cargo bench -p keep --bench engine_bench -- --baseline    keep-gate   # compare
```

### Reference baseline (10-core Apple Silicon, in-process, no network)

| bench | throughput | role |
|---|--:|---|
| `engine_single/set` | ~4.6 M/s | scalar write latency |
| `engine_single/get` | ~4.2 M/s | scalar read latency |
| `engine_single/incr` | ~17 M/s | CAS-free counter |
| `engine_single/cas` | ~8.2 M/s | compare-and-swap |
| `engine_batch100/mset` | ~6.0 M/s | bulk write (claim-check path) |
| `engine_batch100/mget` | ~6.1 M/s | bulk read |
| **`engine_concurrent/set_Nthreads`** | **~13 M/s** | **multi-core SET scaling — the key guard** |

★ The `engine_concurrent` row is the one that matters most: it guards the
multi-core SET-scaling fix (atomic `maxmemory` + arc-swap persistence handle on
the write hot path). A regression that re-introduces a global lock on the write
path collapses this from ~13 M/s to ~3 M/s — treat a drop below **~10 M/s** as a
hard regression.

## 2. Server resource gate — `meter.toml`

`meter.toml` declares ceilings (`[gate] max_peak_rss_mb`, `max_cpu_time_ms`)
enforced against the live keep server under a FIXED load. A breach is a `high`
finding and a non-zero exit.

```bash
cd projects/keep
KEEP_DISABLE_PERSISTENCE=true meter measure \
  --exec ../../target/release/keep --level vitals --duration-cap 30 \
  --drive 'for i in $(seq 1 15); do curl -sf -o /dev/null --max-time 1 \
      http://127.0.0.1:7117/healthz && break; sleep 1; done; \
    ../../target/release/examples/bench_compare --backend keep --ops 200000 \
      --concurrency 1000 --batch 1 --keep-url http://127.0.0.1:7117 --keep-clients 10'
```

Reference vitals for that fixed load: **cpu_time ~6.6 s, peak RSS ~85 MiB**.
Ceilings (`max_peak_rss_mb = 256`, `max_cpu_time_ms = 15000`) leave headroom for
variance while catching memory bloat/leaks and gross per-op CPU regressions.
Verified: lowering the RSS ceiling below the measured value makes the gate exit
non-zero with a `[gate] max_peak_rss_mb breached` finding.

## Notes

- `meter run --target projects/keep` is the composite (test + bench + profile,
  worst-wins) for one-shot CI use.
- Competitor comparison (vs Redis / Dragonfly) is a separate one-off harness:
  `examples/bench_compare.rs` (`--backend redis|dragonfly`). It is NOT part of
  the committed gate (needs external servers); see the keep memory notes.
- meter's auto baseline-store (run → compare → ratchet without a hand-saved
  baseline) is still maturing; until then the criterion `--save-baseline` flow
  above is the throughput ratchet.
