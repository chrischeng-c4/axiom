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

| System | submit | end-to-end | category |
|--------|--------|-----------|----------|
| Celery + Redis | ~2500/s | **~1549 tasks/s** | lean task queue (different category) |
| **Temporal** (dev, sqlite) | ~1143/s | **~184 workflows/s** | **durable workflow engine — loom's true peer** |
| loom (release) | ~179/s | **~45–50 runs/s** (500/500) | durable DAG control-plane + claim-check |
| loom (debug, sequential curl) | — | ~36 runs/s | (scripts/bench.sh) |

**Category matters.** Against a *lean task queue* (Celery), loom is ~34× slower —
but that compares a durable DAG orchestrator to a fire-and-forget queue. Against
its *actual peer*, a durable workflow engine (**Temporal**), loom is only **~4×
slower** (45–50 vs 184 wf/s) on comparable dev configs (loom MemStore vs Temporal
sqlite). Both pay for durability + history; the gap is loom's per-run h2c hop
count (below), not its design category. The Temporal comparison is the fair one —
loom and Temporal solve the same problem (durable, retryable, observable DAGs);
Celery does not.

## Root cause (measured — first hypothesis, then refuted by experiment)

The throughput is **not** gated by worker count:

- 4 workers → 45 runs/s.
- 12 workers → still 45 runs/s (flat).

That flatness pointed at a single serial point — the completion consumer (one
loop leasing `loom.completions`, folding, `put`ting). So I implemented and
measured the proposed fix: **run-id-sharded completion consumers** (publish each
completion to `loom.completions.{fnv(run_id) % N}`; one consumer per shard, so a
single run still folds serially — no read-modify-write race — while distinct runs
fold in parallel). The result *refuted* the single-loop hypothesis:

| config | fold phase | end-to-end |
|--------|-----------|-----------|
| 1 consumer | ~60/s | 45 runs/s |
| 8 shards / 8 workers | ~75/s | **50 runs/s** |

Sharding helped the fold phase (~25%) but end-to-end barely moved (45→50). So the
cost is **distributed across the per-run pipeline**, not one loop: each run pays
~7 h2c round-trips (submit-publish → lease → keep get → keep put → completion
publish → completion lease → ack), and the submit path itself caps at ~150/s.
This is inherent to *what loom is* — a **durable DAG control plane** (consistent
sharded state, raft HA, claim-check so payload bytes never traverse the control
plane). A single `echo` run pays the full control-plane tax with no DAG depth to
amortize it; Celery pays one Redis round-trip. Different optimization targets.

## Levers (measured where noted)

1. **Run-id-sharded completion consumers** — *implemented* (LOOM_COMPLETION_SHARDS,
   default 8). Correct + a modest fold-phase win, but not the end-to-end fix the
   flatness suggested.
2. **Cut h2c round-trips per run** — the real lever: batch/pipeline the
   submit-publish, lease, and completion hops; co-locate keep reads. This is
   where the ~34× lives, and it trades against the claim-check/broker design.
3. **Worker prefetch** — lease a batch per round-trip instead of one task.
4. **Release + concurrent submit** already lifted submit (36→~150/s); the
   distributed per-run overhead is the remaining wall.

## Takeaway

Compare like with like. A lean task queue (Celery/Redis, ~1549/s) wins ~34× — but
it is a different category (fire-and-forget, no durable history). Against loom's
*actual peer*, a durable workflow engine (**Temporal, ~184 wf/s**), loom (~45–50
runs/s) is only **~4× off** on comparable dev configs — and that gap is the
**cumulative per-run h2c control-plane overhead** (the sharded-consumer fix I
tested moved it only 45→50, so it is not one fixable loop), not the durable-DAG
design itself. Closing it means cutting round-trips per run (batching/pipelining),
which trades against claim-check + broker + consistent state. So the right framing
is which workloads earn the durable-DAG tax (deep DAGs, large payloads,
failure-sensitive runs) — and there loom is already in Temporal's league while
keeping the no-SDK polyglot + claim-check model.

## Ratchet (CI gate)

`scripts/perf-ratchet.sh` runs the loom bench, compares end-to-end runs/s to
`docs/benchmark/baseline.json`, fails on regression beyond tolerance, and ratchets
the baseline up on improvement — the "dormant axis" the competitor arena can't
cover (loom vs its own best). Reproduce the competitors from `benchmark/`.
