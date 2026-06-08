"""any() short-circuit reduction — late-hit perf bench.

End-user scenario: `any(xs)` where the truthy element sits near
the end of the list (late hit), the worst-case short-circuit
shape — almost a full scan. CPython routes through builtin_any
+ per-step PyObject_IsTrue; mamba's mb_any inlines the
truthiness probe with a direct compare.

Bounded context (DDD): builtins_bench/any_all.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
# All falsy except the second-to-last slot: forces a near-full scan.
xs = [0] * N
xs[N - 2] = 1
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    if any(xs):
        acc = acc + 1
_t1 = time.perf_counter()

print("any_late_hit_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

diff = acc - ITERS
assert diff == 0, f"checksum mismatch: {acc} - {ITERS} = {diff}"
