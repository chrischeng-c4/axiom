<!-- HANDWRITE-BEGIN gap="missing-generator:logic:58034f48" tracker="#1597" reason="Document the reproducible P0 benchmark profile and its prerequisites." -->
# PgBouncer transaction-pooling baseline

This P0 runner compares the current `pgpool` transaction data path against
PgBouncer with one ephemeral local PostgreSQL backend. It is a reproducible
baseline, not a production performance claim or an enforced performance
ratchet.

## Profile

Both targets use the same intentionally constrained workload:

- PostgreSQL simple-query protocol (`pgbench --protocol simple`); the default
  profile is TPC-B and `--workload select-only` removes TPC-B row-lock
  contention to isolate relay-and-reset throughput
- transaction pooling
- 16 physical backend connections
- 64 clients, 4 pgbench jobs, 30 seconds, TPC-B scale factor 1
- pgpool waits up to 60 seconds for a capped backend lease, matching
  PgBouncer's queueing behavior instead of aborting a client during a
  transient slow host interval
- one freshly initialized, trust-authenticated loopback PostgreSQL database
- `DISCARD ALL` when either target returns a backend connection to its pool;
  the PgBouncer target sets both `server_reset_query = DISCARD ALL` and
  `server_reset_query_always = 1`, so the reset is executed on every
  transaction-pool return rather than merely configured

The runner warms the shared backend, starts both poolers, then gives the target
of each scored leg a five-second target-specific warmup with the same client,
job, protocol, cap, and reset settings. Warmup validation still rejects client
errors, but its TPS and latency are excluded from the JSON and verdict. It then
runs two counterbalanced paired trials sequentially: PgBouncer-first followed
by pgpool-first. Each target therefore receives one first-position and one
second-position 30-second trial without concurrent pgbench traffic against the
shared capped backend. Its JSON preserves raw TPS, average latency, order, and
each pair's `pgpool_over_pgbouncer_tps` ratio, then reports the paired mean.

Each trial carries a winner and the result records whether both pairs agree.
A pgpool win is eligible only when both pairs favor pgpool *and* their ratios
differ by at most 20%; otherwise the result is explicit
`comparison_valid: false`, `pgpool_win_eligible: false`, and
`winner_by_tps: "invalid"`. Conversely, if both clean pairs favor PgBouncer,
the runner emits a PgBouncer verdict even when the loss magnitude varies, so a
candidate is rejected promptly rather than being hidden behind an average.
Mixed pair directions, incomplete clients, and pgbench client errors remain
invalid evidence.

## Run

On macOS, install the local dependencies and build the release binary:

```bash
brew install postgresql@18 pgbouncer
cargo build --release -p pgpool
apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh
```

Use `--dry-run` to inspect the profile without probing commands, creating a
temporary cluster, binding ports, or requiring any installed benchmark tool:

```bash
apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh --dry-run
```

For the contention-free transaction-relay profile used for P0 data-plane work:

```bash
apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh \
  --workload select-only
```

For a debug or custom pgpool binary, pass it explicitly:

```bash
apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh \
  --pgpool-bin target/debug/pgpool
```

To attribute completed pgpool transaction legs without changing their protocol
or pool semantics, add `--phase-telemetry` (and retain the work directory):

```bash
PGPOOL_BENCH_KEEP_WORK_DIR=true \
apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh \
  --workload select-only \
  --phase-telemetry
```

This opt-in mode records only bounded pool aggregates: `acquire`, `relay`, and
`release`, each split into success/failure count and elapsed-time sum. The
retained work directory contains before/after Prometheus snapshots and
`pgpool-*-phase-delta.prom` files for the two pgpool legs. It carries
`comparison_valid: false` and cannot establish a PgBouncer win; use it to pick
the next behavior-changing P0, then rerun the ordinary unsampled comparison.

The runner starts and removes a temporary PostgreSQL data directory itself. It
does not install packages, change a service, or retain benchmark data after it
exits.

For a failed-run diagnosis only, set `PGPOOL_BENCH_KEEP_WORK_DIR=true`; the
runner will print and retain its otherwise-temporary logs and configuration.

## Meter diagnostic

To attribute either pooler's CPU time before making a performance change, build
meter and select the sampled target. The default is pgpool:

```bash
cargo build -p meter-cli --bin meter
apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh \
  --meter-bin target/debug/meter
```

To source-attribute PgBouncer's event-loop path with the same workload, use:

```bash
apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh \
  --meter-bin target/debug/meter \
  --meter-target pgbouncer
```

Meter starts only the selected process and uses its opaque driver to run one
matching 64-client simple-protocol pgbench leg. The runner automatically retains
the temporary directory it prints on exit; it contains `meter-report.json`,
`meter.log`, `.meter/last-report.json`, and `.meter/*.collapsed` stack data.

Sampling changes the selected leg's resource profile, so its comparison JSON
adds `diagnostics.meter_sampled_target` and
`diagnostics.comparison_valid: false`. It does not run the counterbalanced peer
pairs. Treat this run as hotspot evidence only; run the ordinary unsampled
command again to establish competitor-performance evidence.
<!-- HANDWRITE-END -->
