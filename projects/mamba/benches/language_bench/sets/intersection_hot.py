"""set.intersection — overlap-compute perf bench.

End-user scenario: `a.intersection(b)` inside a tight loop, the canonical
overlap primitive that backs every common-tags filter / access-control
permission check / shared-id audience compute / inverted-index AND
clause. CPython routes through PySet_Intersect (C-level probe-the-smaller
loop); mamba's set should hit a native impl through its typed bridge.

Bounded context (DDD): language_bench/sets.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; set.intersection is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

A = set(range(0, 200))
B = set(range(100, 300))
# Mamba ~48x slower; cap ITERS to keep wall under ~3.5s.
ITERS = 10000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    inter = A.intersection(B)
    s = 0
    for v in inter:
        s = s + v
    acc = acc + s
_t1 = time.perf_counter()

print("intersection_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for v in A.intersection(B):
    per_iter = per_iter + v
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
