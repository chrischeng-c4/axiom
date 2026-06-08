"""Hot-loop contextvars.copy_context microbench for #1469 Gate 2.

Predicted regime per scout: pure-Python hot loop.
CPython's `contextvars.copy_context()` walks the current thread's
context-state HAMT and returns a fresh `Context` object — pays for a
C-level dict copy plus the Python call frame on each iter. Mamba
dispatches directly into `mb_contextvars_copy_context` (native
sentinel allocation), so per-call overhead is the constant-time
native handle dispatch.

Workload: 200_000 iters of `contextvars.copy_context()`. Each call
returns a fresh handle in mamba (and a fresh Context in CPython);
the ratio favors mamba because the per-call Python interpreter
overhead dwarfs the native dispatch.

Hoist convention (#2097): bind `contextvars.copy_context` locally to
avoid per-iter module-attr lookup. Same pattern as tempfile/queue
hot-loop bench fixtures.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import contextvars` lines.

# tier: hot-loop
"""

import contextvars

_copy_context = contextvars.copy_context

ITERS = 200_000

acc = 0
for _ in range(ITERS):
    c = _copy_context()
    acc = acc + 1
print("copy_context_hot:", acc)
