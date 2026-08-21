"""min() on int list — builtin perf bench.

End-user scenario: `min(xs)` repeated, the canonical reduction
for floor / lowest / best-case computation. CPython dispatches
through builtin_min with per-compare PyObject_RichCompareBool;
mamba's mb_min scans inline with direct i64 compare.

Bounded context (DDD): builtins_bench/max_min — pairs with
max_int_hot to cover both ends of the reduction.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
xs = [(i * 1103515245 + 12345) & 0xFFFF for i in range(N)]
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = acc + min(xs)
_t1 = time.perf_counter()

print("min_int_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * min(xs)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
