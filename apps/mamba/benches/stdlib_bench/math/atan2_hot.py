"""math.atan2 — two-arg inverse-tan float perf bench.

End-user scenario: `atan2(y, x)` inside a tight loop, the canonical
angle-from-coords primitive that backs every joystick heading compute /
vector-to-bearing conversion / phase angle from complex pair / sprite
rotation toward target. CPython routes through the C math module
(libm); mamba's math should hit a native impl through its typed bridge.

Bounded context (DDD): stdlib_bench/math.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `math.atan2` to a local.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import math
import sys
import time

_atan2 = math.atan2

N = 1000
STEP = 0.01
# Mamba ~24x slower; cap ITERS so wall stays under ~2s.
ITERS = 300

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    y = 0.0
    x = 1.0
    for inner in range(N):
        s = s + _atan2(y, x)
        y = y + STEP
    acc = acc + s
_t1 = time.perf_counter()

print("atan2_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0.0
y = 0.0
x = 1.0
for k in range(N):
    per_iter = per_iter + _atan2(y, x)
    y = y + STEP
expected = ITERS * per_iter
diff = acc - expected
assert math.isclose(acc, expected, rel_tol=1e-9), f"checksum mismatch: {acc} vs {expected} (diff={diff})"
