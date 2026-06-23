# loom frontier benchmark (#127)

Where loom sits versus a mature task system on the **same machine**, plus the
root-cause of loom's current ceiling and the levers to raise it.

## Setup

- Machine: this dev box (Apple Silicon, 10 cores).
- Unit of work: one trivial `echo` task / single-node run.
- N = 500, 4 workers, **async/concurrent submit** on both sides (fair submit path).
- loom: release build, real `relay` (broker) + `keep` (store) over h2c, claim-check.
- Celery 5.6 + Redis 7 broker/back-end, `--concurrency=4`.
- Reproduce: `scripts/bench.sh` (loom, self-contained) and the Celery app in the
  issue thread; the loom side here used a concurrent (thread-pool) submitter.

## Results

| System | submit | end-to-end | model |
|--------|--------|-----------|-------|
| Celery + Redis | ~2500/s | **~1549 tasks/s** | direct task queue |
| loom (release) | ~179/s | **~45 runs/s** (500/500) | DAG control-plane + claim-check |
| loom (debug, sequential curl) | — | ~36 runs/s | (scripts/bench.sh) |

loom is ~34× slower than Celery on this micro-workload.

## Root cause (verified, not hypothesized)

The ceiling is the **single, serial completion consumer** in the controller, not
the worker count:

- 4 workers → 45 runs/s.
- **12 workers → still 45 runs/s** (flat).

Every run's terminal completion funnels through one consumer that leases
`loom.completions` one entry at a time, folds it into the DAG, and `put`s the
run — serially. At ~45/s that single loop is the bottleneck; adding workers (or
submit concurrency) cannot move it.

This is inherent to *what loom is*, not a bug: loom is a **durable DAG control
plane** (sharded consistent state, HA via raft, claim-check so payload bytes
never traverse the control plane). A single `echo` run pays the full
control-plane tax — submit → relay publish → lease → keep get → keep put → relay
completion publish → completion lease → fold → store — with no DAG depth to
amortize it over. Celery pays one Redis round-trip. They optimize different
things: Celery for raw fan-out throughput, loom for durable multi-step
orchestration with failover.

## Levers to raise loom's ceiling (future #127 work)

1. **Run-id-sharded completion consumers** — the highest-value fix. Parallelize
   completion folding across a pool keyed by `run_id` (so a single run's nodes
   still fold serially, avoiding the read-modify-write race a naive parallel
   consumer would hit). Should scale near-linearly with shards until keep/relay
   saturate.
2. **Worker prefetch** — lease a batch per round-trip instead of one task.
3. **Fewer h2c hops** — co-locate keep result-ref reads; batch the completion
   publish/ack.
4. **Release + batched submit** already help the submit path (179 vs 36/s); the
   fold path is the remaining wall.

## Takeaway

For fine-grained, independent tasks, a lean queue (Celery/Redis) wins decisively.
loom's value is durable, HA, claim-check DAG orchestration; its current
single-run throughput is gated by a serial fold loop that is a known, bounded
optimization target (run-id-sharded consumers), not an architectural limit.
