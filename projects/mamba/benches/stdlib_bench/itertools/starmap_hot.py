"""itertools.starmap — tuple-spread map perf bench.

End-user scenario: `starmap(operator_op, rows_of_arg_tuples)` inside a
tight loop, the canonical apply-each-row-as-args primitive that backs
every row-wise computation over CSV-like tuples / pairwise difference
over (x,y) coordinate pairs / vectorized polynomial eval over
(coef, x) pairs. CPython routes through itertools_starmap_impl
(C-level zip-then-call); mamba's itertools should hit a native impl
through its typed bridge.

Distinct from `chain_two_lists_hot.py` (splice) and `accumulate_hot.py`
(scan). starmap is the bridge between zip-shaped data and an existing
binary fn — the *args spread happens inside the C loop, not at the
Python level.

Bounded context (DDD): stdlib_bench/itertools.

Tier: compute (with per-call new-iterator + per-yield small-int allocation).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `starmap` is a module-level free fn; safe to hoist locally.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import itertools
import sys
import time


def add2(a, b):
    return a + b


_starmap = itertools.starmap
PAIRS = ((1, 2), (3, 4), (5, 6), (7, 8), (9, 10),
         (11, 12), (13, 14), (15, 16), (17, 18), (19, 20),
         (21, 22), (23, 24), (25, 26), (27, 28), (29, 30))
ITERS = 10000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for v in _starmap(add2, PAIRS):
        s = s + v
    acc = acc + s
_t1 = time.perf_counter()

print("starmap_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for v in itertools.starmap(add2, PAIRS):
    per_iter = per_iter + v
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
