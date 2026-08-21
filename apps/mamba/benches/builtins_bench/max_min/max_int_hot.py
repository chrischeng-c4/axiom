"""max() on int list — builtin perf bench.

End-user scenario: `max(xs)` repeated, the canonical reduction for
ceiling / hottest / worst-case computation. CPython dispatches
through builtin_max with per-compare PyObject_RichCompareBool;
mamba's mb_max scans inline with direct i64 compare.

Bounded context (DDD): builtins_bench/max_min — groups max/min
together since they share the same reduction pattern.

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
    acc = acc + max(xs)
_t1 = time.perf_counter()

print("max_int_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Reference using max() once more — both runtimes see the same input.
expected = ITERS * max(xs)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
