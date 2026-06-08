"""Hot-loop bench for `threading.Lock` / `threading.RLock` /
`threading.Event` / `threading.Thread` / `threading.Condition` /
`threading.Semaphore` / `threading.Timer` / `threading.current_thread`
module-attribute reads (#1438).

End-user scenario: hot concurrency-orchestration code that resolves
synchronization-primitive constructors and the current-thread accessor
off the `threading` module on every call site — e.g. a worker-pool
loop that calls `threading.Lock()` to mint a per-task mutex,
`threading.RLock()` for re-entrant guards around recursive callbacks,
`threading.Event()` for cancellation signaling, `threading.Thread`
to spawn a follow-up worker, `threading.Condition()` to gate a
producer/consumer rendezvous, `threading.Semaphore()` for bounded
concurrency, `threading.Timer` for deferred callbacks, and
`threading.current_thread()` to tag log records. The canonical
hot-path idiom is to read those names directly off the `threading`
module on every call rather than caching a local — keeps the call
site robust against late-binding patterns (test monkey-patching,
thread-factory plugins, runtime backend-swap fixtures). That
per-iter module-attribute octuple read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x —
CPython's `threading.Lock` / `threading.RLock` / `threading.Event` /
`threading.Thread` / `threading.Condition` / `threading.Semaphore` /
`threading.Timer` / `threading.current_thread` are top-level
module-dict probes returning the canonical class/function objects on
3.12). Mamba's shim returns the same identity-stable callables
directly from a dense constant table in the `threading`
module-attribute resolver, short-circuiting CPython's module-dict
probe chain for read-only sync-primitive sentinels.

Workload: 10_000 paired reads of `threading.Lock`, `threading.RLock`,
`threading.Event`, `threading.Thread`, `threading.Condition`,
`threading.Semaphore`, `threading.Timer`, and
`threading.current_thread` per iteration, compared by identity
(`is`) against the hoisted baseline references taken once before
the loop. The accumulator increments when all eight reads resolve
to the identical callable objects; a misread (different identity /
wrong binding) would immediately fail the correctness assert and
dead-code elimination of any read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import threading as _th

# Hoist baseline references to the canonical sync-primitive callables
# once before the loop. The hot path re-reads the module attribute on
# every iter so the bench actually exercises the module-attribute
# resolver — the `is` compare against the hoisted baseline is the
# correctness probe.
_LOCK_BASELINE = _th.Lock
_RLOCK_BASELINE = _th.RLock
_EVENT_BASELINE = _th.Event
_THREAD_BASELINE = _th.Thread
_CONDITION_BASELINE = _th.Condition
_SEMAPHORE_BASELINE = _th.Semaphore
_TIMER_BASELINE = _th.Timer
_CURRENT_THREAD_BASELINE = _th.current_thread

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    l = _th.Lock
    r = _th.RLock
    e = _th.Event
    t = _th.Thread
    c = _th.Condition
    s = _th.Semaphore
    tm = _th.Timer
    ct = _th.current_thread
    # Accumulator readback prevents DCE — every iteration must
    # resolve to the identical callable objects bound at the
    # `threading.Lock` / `threading.RLock` / `threading.Event` /
    # `threading.Thread` / `threading.Condition` /
    # `threading.Semaphore` / `threading.Timer` /
    # `threading.current_thread` module slots.
    if (l is _LOCK_BASELINE
            and r is _RLOCK_BASELINE
            and e is _EVENT_BASELINE
            and t is _THREAD_BASELINE
            and c is _CONDITION_BASELINE
            and s is _SEMAPHORE_BASELINE
            and tm is _TIMER_BASELINE
            and ct is _CURRENT_THREAD_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical
# sync-primitive callables via the module-attribute resolver.
# acc == ITERS or we have a regression in mamba's threading
# module-attribute table.
assert acc - ITERS == 0, f"threading module-attribute read acc drift: acc={acc} expected={ITERS}"
print("sync_type_read_hot:", acc)
