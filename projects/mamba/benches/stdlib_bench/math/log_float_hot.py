"""math.log — natural-log perf bench.

End-user scenario: `math.log(x)` inside a tight numeric loop,
the canonical per-element transform that backs every entropy /
log-likelihood / information-theoretic accumulator. CPython
routes through math_log -> libm log; mamba's math.log lowers
to a native f64 log op on the JIT typed path.

Bounded context (DDD): stdlib_bench/math.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `log` to a local before the hot loop.
"""

import math
import sys
import time

_log = math.log

N = 1000
xs = [float(i + 1) for i in range(N)]
ITERS = 1000

acc = 0.0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        acc = acc + _log(x)
_t1 = time.perf_counter()

print("log_float_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref_per_iter = 0.0
for x in xs:
    ref_per_iter = ref_per_iter + _log(x)
expected = ITERS * ref_per_iter
assert math.isclose(acc, expected, rel_tol=1e-9, abs_tol=1e-3), (
    f"checksum mismatch: {acc} vs {expected}"
)
