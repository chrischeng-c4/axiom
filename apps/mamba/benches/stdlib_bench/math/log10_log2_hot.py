"""math.log10 / math.log2 — base-10 + base-2 log perf bench.

End-user scenario: `math.log10(bytes)` for KB/MB/GB scale-bucketers
and `math.log2(n)` for bit-width / tree-depth / hash-table-power-of-2
computations, inside a tight loop. Backs every storage scale-axis
labeler / bit-bucket counter / Bloom-filter-size sizer / tree-height
estimator. CPython routes through math_log10 / math_log2 (C-level
libm); mamba's math should hit a native FFI thunk through its typed
bridge.

Distinct from `log_hot.py` (covers `math.log` natural log).

Bounded context (DDD): stdlib_bench/math.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `math.log10` / `math.log2` to locals — module-level
free fns, safe to hoist.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import math
import sys
import time

_log10 = math.log10
_log2 = math.log2

N = 200
ITERS = 5000

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    for i in range(N):
        x = float(i + 1)
        s = s + _log10(x) + _log2(x)
    acc = acc + s
_t1 = time.perf_counter()

print("log10_log2_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Use isclose for FP checksum — sums of transcendentals have small
# FP-association drift but should agree to ~1e-9 relative.
per_iter = 0.0
for i in range(N):
    x = float(i + 1)
    per_iter = per_iter + math.log10(x) + math.log2(x)
expected = ITERS * per_iter
assert math.isclose(acc, expected, rel_tol=1e-9), f"checksum drift: {acc} vs {expected}"
