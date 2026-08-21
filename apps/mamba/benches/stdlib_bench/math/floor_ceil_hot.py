"""math.floor / math.ceil — integer-truncation perf bench.

End-user scenario: `math.floor(x)` / `math.ceil(x)` inside a tight
loop, the canonical FP-to-int rounder primitive that backs every
bucket-index assigner / page-count computation / tile-coverage
calculator / grid-snap quantizer. CPython routes through math_floor
/ math_ceil (C-level libm + PyLong build); mamba's math should hit
a native FFI thunk through its typed bridge.

Bounded context (DDD): stdlib_bench/math.

Tier: compute (with PyLong allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `math.floor` / `math.ceil` module attrs to locals —
they ARE module-level free fns (NOT bound methods), so the hoist
is safe.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars. `import a, b` only binds `a` under mamba — use
separate `import` lines.
"""

import math
import sys
import time

_floor = math.floor
_ceil = math.ceil

N = 200
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(N):
        x = float(i) * 0.37
        s = s + _floor(x) + _ceil(x)
    acc = acc + s
_t1 = time.perf_counter()

print("floor_ceil_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(N):
    x = float(i) * 0.37
    per_iter = per_iter + math.floor(x) + math.ceil(x)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
