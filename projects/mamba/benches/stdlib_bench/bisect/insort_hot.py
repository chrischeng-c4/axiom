"""bisect.insort — sorted-insert perf bench.

End-user scenario: `bisect.insort(sorted_queue, ts)` inside a tight loop,
the canonical maintain-sorted-on-insert primitive that backs every
sorted timeline of events / online-quantile rolling window / leaderboard
incremental-update / sorted index of pending tasks. CPython routes
through bisect_insort_right (C-level binary search + list.insert);
mamba's bisect should hit a native impl through its typed bridge.

Distinct from `bisect_left_hot.py` which is pure search (no mutate);
insort exercises the bisect+memmove path that dominates real workloads.

Bounded context (DDD): stdlib_bench/bisect.

Tier: compute (with per-call list-grow + memmove on insert position).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `insort` is a module-level free fn; safe to hoist locally.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.

Each outer iter REBUILDS the working list to keep length stable; the
hot path is the insort itself, not unbounded growth.
"""

import bisect
import sys
import time

_insort = bisect.insort
SEED = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100,
        110, 120, 130, 140, 150, 160, 170, 180, 190, 200]
TO_INSERT = (5, 25, 45, 65, 85, 105, 125, 145, 165, 185, 205, 75)
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    xs = list(SEED)
    for v in TO_INSERT:
        _insort(xs, v)
    acc = acc + len(xs)
_t1 = time.perf_counter()

print("insort_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(SEED) + len(TO_INSERT)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
