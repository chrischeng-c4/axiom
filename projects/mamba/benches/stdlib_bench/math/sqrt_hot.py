"""math.sqrt — scalar square-root perf bench.

End-user scenario: `math.sqrt(x)` inside a tight loop, the canonical
scalar-math primitive that backs every L2-norm / distance / RMS /
gaussian-step kernel that doesn't reach for NumPy. CPython routes
through C-level math.sqrt (libm); mamba's math should hit a native
impl through its typed bridge.

Bounded context (DDD): stdlib_bench/math.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `sqrt` to a local.
"""

import math
import sys
import time

_sqrt = math.sqrt

N = 10000
xs = [float(i + 1) for i in range(N)]
ITERS = 100

acc = 0.0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = 0.0
    for x in xs:
        s = s + _sqrt(x)
    acc = acc + s
_t1 = time.perf_counter()

print("sqrt_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref = 0.0
for x in xs:
    ref = ref + _sqrt(x)
expected = ITERS * ref
assert math.isclose(acc, expected, rel_tol=1e-9, abs_tol=1e-3), (
    f"checksum mismatch: {acc} vs {expected}"
)
