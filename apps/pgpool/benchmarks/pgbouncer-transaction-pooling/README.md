<!-- HANDWRITE-BEGIN gap="missing-generator:logic:58034f48" tracker="#1597" reason="Document the reproducible P0 benchmark profile and its prerequisites." -->
# PgBouncer transaction-pooling baseline

This P0 runner compares the current `pgpool` transaction data path against
PgBouncer with one ephemeral local PostgreSQL backend. It is a reproducible
baseline, not a production performance claim or an enforced performance
ratchet.

## Profile

Both targets use the same intentionally constrained workload:

- PostgreSQL simple-query protocol (`pgbench --protocol simple`)
- transaction pooling
- 16 physical backend connections
- 64 clients, 4 pgbench jobs, 30 seconds, TPC-B scale factor 1
- pgpool waits up to 60 seconds for a capped backend lease, matching
  PgBouncer's queueing behavior instead of aborting a client during a
  transient slow host interval
- one freshly initialized, trust-authenticated loopback PostgreSQL database
- `DISCARD ALL` when PgBouncer returns a backend connection to its pool

The runner warms the shared backend before measurement and then measures the
two targets sequentially. Its JSON includes raw TPS, average latency, and the
explicit `pgpool_over_pgbouncer_tps` ratio; it deliberately does not encode a
pass/fail threshold. It rejects a target that cannot establish all 64 declared
clients or logs a pgbench client error, so a partial-workload run cannot be
compared as a valid result.

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

For a debug or custom pgpool binary, pass it explicitly:

```bash
apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh \
  --pgpool-bin target/debug/pgpool
```

The runner starts and removes a temporary PostgreSQL data directory itself. It
does not install packages, change a service, or retain benchmark data after it
exits.

For a failed-run diagnosis only, set `PGPOOL_BENCH_KEEP_WORK_DIR=true`; the
runner will print and retain its otherwise-temporary logs and configuration.
<!-- HANDWRITE-END -->
