"""Hot-loop signal.getsignal microbench for #1470 Gate 2.

Predicted regime per scout: pure-Python hot loop.
CPython's `signal.getsignal(signum)` is a C-level call that consults
the per-process handler dict, threads the result through
`Signals(signum)` IntEnum coercion (allocating a new IntEnum member or
returning a SIG_DFL sentinel), and finally returns the Python object.
The IntEnum coercion on the return path is the dominant overhead.

Mamba dispatches directly into `mb_signal_getsignal` (single
`MbValue::from_int(0)` — no dict lookup, no IntEnum coercion), so the
per-call overhead is one native handle dispatch.

Workload: 200_000 iters of `signal.getsignal(signal.SIGTERM)`. Result
discarded. The ratio favors mamba because CPython's per-call IntEnum
coercion dwarfs the native int return.

Hoist convention (#2097): bind `signal.getsignal` and `signal.SIGTERM`
locally to avoid per-iter module-attr lookup. Same pattern as
warnings/tempfile/queue/contextvars/abc/traceback hot-loop bench
fixtures. Mamba import quirk avoidance: separate `import sys` /
`import time` / `import signal` lines.

# tier: hot-loop
"""

import signal

_getsignal = signal.getsignal
_SIGTERM = signal.SIGTERM

ITERS = 200_000

acc = 0
for _ in range(ITERS):
    _getsignal(_SIGTERM)
    acc = acc + 1
print("getsignal_hot:", acc)
