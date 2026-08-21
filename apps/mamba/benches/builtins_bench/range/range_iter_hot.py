"""range() iteration — builtin perf bench.

End-user scenario: empty-body for-loop over range(N), the
canonical iteration overhead probe. Measures pure loop dispatch
(no body work) — the floor cost of every `for i in range(N)`
loop. Mamba's JIT lowers tight numeric ranges to a native
counter loop; CPython dispatches FOR_ITER bytecode + range
iterator advance per step.

Bounded context (DDD): builtins_bench/range.

Tier: compute.

#2105: print of `count` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

ITERS = 10_000_000

count = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    count = count + 1
_t1 = time.perf_counter()

print("range_iter_hot:", count)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

diff = count - ITERS
assert diff == 0, f"checksum mismatch: {count} - {ITERS} = {diff}"
