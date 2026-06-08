"""Hot-loop bench for `concurrent.futures.Future` /
`concurrent.futures.Executor` / `concurrent.futures.ThreadPoolExecutor`
module-attribute reads (#1473).

End-user scenario: hot dispatch / orchestration code that
introspects the executor type family on every submission — e.g.
`isinstance(ex, concurrent.futures.Executor)` to gate dispatch,
`isinstance(f, concurrent.futures.Future)` to recognise pending
results, `isinstance(ex, concurrent.futures.ThreadPoolExecutor)` to
pick a thread-vs-process branch. The canonical hot-path idiom is to
read those type names directly off the `concurrent.futures` module
each call rather than caching a local — keeps the call site robust
against late-binding patterns (test monkey-patching, plugin reloads,
backend-swap fixtures). That per-iter module-attribute triple-read
is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x —
CPython's `concurrent.futures.Future` /
`concurrent.futures.Executor` /
`concurrent.futures.ThreadPoolExecutor` are top-level module-dict
probes returning the class objects on 3.12). Mamba's shim returns
the same callable objects directly from a dense constant table in
the `concurrent.futures` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only class sentinels.

Workload: 10_000 paired reads of `concurrent.futures.Future`,
`concurrent.futures.Executor`, and
`concurrent.futures.ThreadPoolExecutor` per iteration, compared by
identity (`is`) against the hoisted baseline references taken once
before the loop. The accumulator increments when all three reads
resolve to the identical class objects; a misread (different
identity / wrong binding) would immediately fail the correctness
assert and dead-code elimination of any read would leave
`acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import concurrent.futures as _cf

# Hoist baseline references to the canonical class objects once
# before the loop. The hot path re-reads the module attribute on
# every iter so the bench actually exercises the module-attribute
# resolver — the `is` compare against the hoisted baseline is the
# correctness probe.
_FUTURE_BASELINE = _cf.Future
_EXECUTOR_BASELINE = _cf.Executor
_TPE_BASELINE = _cf.ThreadPoolExecutor

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    f = _cf.Future
    e = _cf.Executor
    tpe = _cf.ThreadPoolExecutor
    # Accumulator readback prevents DCE — every iteration must
    # resolve to the identical class objects bound at the
    # `concurrent.futures.Future` / `concurrent.futures.Executor` /
    # `concurrent.futures.ThreadPoolExecutor` module slots.
    if (f is _FUTURE_BASELINE
            and e is _EXECUTOR_BASELINE
            and tpe is _TPE_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical class
# objects via the module-attribute resolver. acc == ITERS or we have
# a regression in mamba's concurrent.futures module-attribute table.
assert acc - ITERS == 0, f"concurrent.futures module-attribute read acc drift: acc={acc} expected={ITERS}"
print("executor_type_read_hot:", acc)
