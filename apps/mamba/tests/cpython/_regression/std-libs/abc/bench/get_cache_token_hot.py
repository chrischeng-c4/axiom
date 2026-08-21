"""Hot-loop abc.get_cache_token microbench for #1447 Gate 2.

Predicted regime per scout: pure-Python hot loop.
CPython's `abc.get_cache_function()` is pure Python in `Lib/abc.py` —
each call walks a Python call frame and returns the module-level
`_abc_invalidation_counter` int. Mamba dispatches directly into
`mb_abc_get_cache_token` (native AtomicU64 fetch_add), so per-call
overhead is the constant-time native handle dispatch.

Workload: 200_000 iters of `abc.get_cache_token()`. Each call returns
a fresh int. The ratio favors mamba because the per-call Python
interpreter overhead dwarfs the native AtomicU64 increment.

Hoist convention (#2097): bind `abc.get_cache_token` locally to
avoid per-iter module-attr lookup. Same pattern as
tempfile/queue/contextvars hot-loop bench fixtures.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import abc` lines.

# tier: hot-loop
"""

import abc

_get_cache_token = abc.get_cache_token

ITERS = 200_000

acc = 0
for _ in range(ITERS):
    t = _get_cache_token()
    acc = acc + 1
print("get_cache_token_hot:", acc)
