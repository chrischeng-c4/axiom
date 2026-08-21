"""Hot-loop bench for `multiprocessing.Process` /
`multiprocessing.Queue` / `multiprocessing.cpu_count` /
`multiprocessing.current_process` module-attribute reads (#1476).

End-user scenario: parallel-compute glue (worker pools, fan-out
dispatch scripts, batch ingesters, CI runners) typically reads
`multiprocessing.Process` / `multiprocessing.Queue` /
`multiprocessing.cpu_count` / `multiprocessing.current_process`
on every entry-point site rather than caching a local alias.
Wrapper code that spawns workers via
`p = multiprocessing.Process(target=fn)` or sizes a pool via
`n = multiprocessing.cpu_count()` re-resolves these names through
the module's attribute table on each call site. That per-call
module-attribute quad-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `multiprocessing.Process` is a top-level class on 3.12,
`multiprocessing.Queue` / `multiprocessing.cpu_count` /
`multiprocessing.current_process` are bound methods on the
default-context object). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table in
the `multiprocessing` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only multiprocessing
sentinels.

Workload: 20_000 paired reads of `Process`, `Queue`, `cpu_count`,
and `current_process` per iteration (ITERS scaled to 20_000 so
4 attrs x 20k = ~80k attr-reads per run, matching the per-spawn
budget of the 8-attr fixtures at 10_000 iters, the 2-attr fixtures
at 40_000 iters, and the 1-attr fixtures at 80_000 iters). All
four values are re-resolved from the `multiprocessing`
module-attribute table on every iter (not hoisted to a local) and
identity-compared against the hoisted baseline references; the
accumulator increments when all four reads resolve to identical
objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import multiprocessing as _mp

_PROCESS_BASELINE = _mp.Process
_QUEUE_BASELINE = _mp.Queue
_CPU_COUNT_BASELINE = _mp.cpu_count
_CURRENT_PROCESS_BASELINE = _mp.current_process

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _mp.Process
    b = _mp.Queue
    c = _mp.cpu_count
    d = _mp.current_process
    if (a is _PROCESS_BASELINE
            and b is _QUEUE_BASELINE
            and c is _CPU_COUNT_BASELINE
            and d is _CURRENT_PROCESS_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"multiprocessing module-attribute read acc drift: acc={acc} expected={ITERS}"
print("multiprocessing_type_read_hot:", acc)
