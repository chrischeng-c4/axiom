"""Hot-loop warnings.simplefilter microbench for #1445 Gate 2.

Predicted regime per scout: pure-Python hot loop.
CPython's `warnings.simplefilter()` is pure Python in `Lib/warnings.py` —
each call walks a Python call frame, validates the `action` against the
allowed set, asserts on `category`, and `_filters_mutated()`s the global
list. Mamba dispatches directly into `mb_warnings_simplefilter` (native
push onto a thread-local Vec<String>), so per-call overhead is the
constant-time native handle dispatch.

Workload: 200_000 iters of `warnings.simplefilter("ignore")`. Each call
returns None. The ratio favors mamba because the per-call Python
interpreter overhead (assert, set membership test, list-mutation
notification) dwarfs the native Vec push.

Hoist convention (#2097): bind `warnings.simplefilter` locally to
avoid per-iter module-attr lookup. Same pattern as
tempfile/queue/contextvars/abc hot-loop bench fixtures.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import warnings` lines.

# tier: hot-loop
"""

import warnings

_simplefilter = warnings.simplefilter

ITERS = 200_000

acc = 0
for _ in range(ITERS):
    _simplefilter("ignore")
    acc = acc + 1
print("simplefilter_hot:", acc)
