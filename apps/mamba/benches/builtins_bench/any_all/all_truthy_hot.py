"""all() short-circuit reduction — builtin perf bench.

End-user scenario: `all(xs)` over a fully-truthy list, the
canonical boolean reduction that backs every invariant /
precondition check. CPython routes through builtin_all + per-step
PyObject_IsTrue; mamba's mb_all inlines the truthiness probe with
a direct compare.

Bounded context (DDD): builtins_bench/any_all — groups the
short-circuit reductions together since they share the same
iteration + early-exit shape.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
# Fully truthy (no early exit) — exercises the worst-case full-scan path.
xs = [1] * N
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    if all(xs):
        acc = acc + 1
_t1 = time.perf_counter()

print("all_truthy_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

diff = acc - ITERS
assert diff == 0, f"checksum mismatch: {acc} - {ITERS} = {diff}"
