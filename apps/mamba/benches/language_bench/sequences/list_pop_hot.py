"""list.pop — tail-pop drain perf bench.

End-user scenario: tight loop `while stack: top = stack.pop()`, the
canonical LIFO drain primitive that backs every DFS traversal /
parser shift-reduce machine / undo-stack consumer / backtracking
search frame popper. CPython routes through list_pop_impl
(C-level array shrink + last-elem return, O(1) at tail); mamba's
list should hit a native impl through its typed bridge.

Distinct from `list_append_loop_hot.py` (push variant) — pop is
the inverse hot path. Drain-from-tail is O(1) per call (no shift).

Bounded context (DDD): language_bench/sequences.

Tier: compute (per-call last-elem return + ob_size decrement; no
alloc on shrink — list keeps capacity).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `pop` is a list method; DO NOT hoist `_pop = lst.pop` —
bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

SEED = (10, 20, 30, 40, 50, 60, 70, 80, 90, 100,
        110, 120, 130, 140, 150, 160, 170, 180, 190, 200)
SEED_LEN = len(SEED)
ITERS = 10000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    lst = list(SEED)
    s = 0
    for inner in range(SEED_LEN):
        s = s + lst.pop()
    acc = acc + s
_t1 = time.perf_counter()

print("list_pop_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
ref = list(SEED)
for inner in range(SEED_LEN):
    per_iter = per_iter + ref.pop()
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
