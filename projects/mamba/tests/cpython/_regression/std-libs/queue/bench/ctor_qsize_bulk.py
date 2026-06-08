"""Bulk queue.Queue construction + qsize cycle (Gate 2 perf gate #1472).

Predicted regime: balanced. Per-iter cost is one `Queue()` handle alloc
plus one `qsize()` method dispatch. Mamba's `make_handle` is a
`thread_local` `HashMap` insert + integer counter bump (no locking);
CPython routes `__init__` through `threading.Lock()` +
`collections.deque()` construction + two `threading.Event()`
instantiations, then `qsize()` through the Lock's `acquire`/`release`.
The constructor is the dominant cost — that's where mamba's
locking-free integer-handle table shines.

Workload: 200000 iters of `(construct + qsize)`. Wall target stays
~50-250ms — small enough for fast CI, large enough to drown jitter.

Hoist convention (#2097): bind `queue.Queue` locally to avoid
per-iter module-attr lookup.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import queue` lines.

# tier: balanced
"""

import queue

_Queue = queue.Queue
ITERS = 200000

acc = 0
for _ in range(ITERS):
    q = _Queue()
    acc += q.qsize()
print("ctor_qsize_bulk:", acc)
