"""Bulk queue.Queue put/get cycle (Task #70, Wave-6 ship #3).

Predicted regime per scout: balanced (per-cycle integer-handle method
dispatch + VecDeque push/pop). Wall target >=2.0x — CPython
queue.Queue routes every .put/.get through threading.Lock acquire/
release + Condvar notify even in single-threaded use; mamba routes
through a thread_local handle table and class.rs int-handle method
dispatch branch with no locking. The per-cycle work is dominated by
the method-dispatch hop, so this exercises the class.rs branch added
at this revision.

Workload: 10000 put/get cycles × 10 iters. Each cycle puts one int
then gets one int — keeps qsize=1 throughout so VecDeque stays small
and the hot loop is pure dispatch + small-int handle alloc.

Per scout sequencing: queue is rewrite from dict-backed to
integer-handle (#1472); this fixture pairs with queue_mod.rs
registering 8 dispatchers + 2 exception class shells (queue.Empty,
queue.Full) at the same revision. class.rs is wired with method
dispatch for handle-int receivers (.put / .get / .empty / .qsize /
.full / .task_done / .join).

Hoist convention (#2097): bind `queue.Queue` locally to avoid
per-iter module-attr lookup.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import queue` lines.

# tier: balanced
"""

import queue

_Queue = queue.Queue

q = _Queue()
ITERS = 10
N = 10000

acc = 0
for _ in range(ITERS):
    for i in range(N):
        q.put(i)
        v = q.get()
        if v == i:
            acc += 1
print("put_get_bulk:", acc)
