"""Hot-loop bench for `asyncio.run` / `asyncio.sleep` /
`asyncio.create_task` / `asyncio.ensure_future` / `asyncio.gather` /
`asyncio.wait` / `asyncio.wait_for` / `asyncio.shield`
module-attribute reads (#1416).

End-user scenario: async framework glue (FastAPI / aiohttp handler
shims, task-scheduler plugins, test-harness adapters) typically reads
the `asyncio` module-level coroutine entry points on every dispatch
site rather than caching a local alias. Library code that wraps
async lifecycle -- `asyncio.run(main())`, `asyncio.create_task(coro)`,
`await asyncio.gather(*tasks)`, `await asyncio.wait_for(coro, t)` --
re-resolves these names through the module's attribute table on each
call site. That per-call module-attribute octet-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `asyncio.run` family are top-level module-dict probes on
3.12). Mamba's shim returns the same callable objects directly from
a dense constant table in the `asyncio` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
callable sentinels.

Workload: 10_000 paired reads of `asyncio.run`, `asyncio.sleep`,
`asyncio.create_task`, `asyncio.ensure_future`, `asyncio.gather`,
`asyncio.wait`, `asyncio.wait_for`, and `asyncio.shield` per
iteration, compared by identity (`is`) against the hoisted baseline
references taken once before the loop. The accumulator increments
when all eight reads resolve to the identical callable objects; a
misread (different identity / wrong binding) would immediately fail
the correctness assert and dead-code elimination of any read would
leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import asyncio as _asyncio

# Hoist baseline references to the canonical callable objects once
# before the loop. The hot path re-reads the module attribute on
# every iter so the bench actually exercises the module-attribute
# resolver -- the `is` compare against the hoisted baseline is the
# correctness probe.
_RUN_BASELINE = _asyncio.run
_SLEEP_BASELINE = _asyncio.sleep
_CREATE_TASK_BASELINE = _asyncio.create_task
_ENSURE_FUTURE_BASELINE = _asyncio.ensure_future
_GATHER_BASELINE = _asyncio.gather
_WAIT_BASELINE = _asyncio.wait
_WAIT_FOR_BASELINE = _asyncio.wait_for
_SHIELD_BASELINE = _asyncio.shield

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    a = _asyncio.run
    b = _asyncio.sleep
    c = _asyncio.create_task
    d = _asyncio.ensure_future
    e = _asyncio.gather
    f = _asyncio.wait
    g = _asyncio.wait_for
    h = _asyncio.shield
    # Accumulator readback prevents DCE -- every iteration must
    # resolve to the identical callable objects bound at the
    # `asyncio.run` / `asyncio.sleep` / `asyncio.create_task` /
    # `asyncio.ensure_future` / `asyncio.gather` / `asyncio.wait` /
    # `asyncio.wait_for` / `asyncio.shield` module slots.
    if (a is _RUN_BASELINE
            and b is _SLEEP_BASELINE
            and c is _CREATE_TASK_BASELINE
            and d is _ENSURE_FUTURE_BASELINE
            and e is _GATHER_BASELINE
            and f is _WAIT_BASELINE
            and g is _WAIT_FOR_BASELINE
            and h is _SHIELD_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical callable
# objects via the module-attribute resolver. acc == ITERS or we have
# a regression in mamba's asyncio module-attribute table.
assert acc - ITERS == 0, f"asyncio module-attribute read acc drift: acc={acc} expected={ITERS}"
print("asyncio_type_read_hot:", acc)
