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

For fine-grained, independent tasks, a lean queue (Celery/Redis) wins decisively
(~34×). loom's value is durable, HA, claim-check DAG orchestration. Its single-run
throughput is gated not by one fixable loop (the sharded-consumer fix I tested
moved it only 45→50 runs/s) but by the **cumulative per-run h2c control-plane
overhead** — the price of claim-check + a durable broker + consistent state. The
honest lesson: closing the gap means cutting round-trips per run (batching), which
trades directly against the claim-check/durability design — so the right question
is not "how do we match Celery's throughput" but "for which workloads is the
durable-DAG tax worth it" (deep DAGs, large payloads, failure-sensitive runs),
where the per-run overhead amortizes and Celery would need bespoke plumbing.
