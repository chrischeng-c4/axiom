"""math.sin/math.cos — transcendental float perf bench.

End-user scenario: `sin(x) + cos(x)` inside a tight loop, the canonical
trig primitive that backs every signal-synth tone generator / rotation
matrix build / FFT twiddle factor / phasor projection. CPython routes
through the C math module (libm); mamba's math should hit a native impl
through its typed bridge.

Bounded context (DDD): stdlib_bench/math.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `math.sin` and `math.cos` to locals.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import math
import sys
import time

_sin = math.sin
_cos = math.cos

N = 1000
STEP = 0.001
ITERS = 500

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    x = 0.0
    for inner in range(N):
        s = s + _sin(x) + _cos(x)
        x = x + STEP
    acc = acc + s
_t1 = time.perf_counter()

print("sin_cos_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0.0
x = 0.0
for k in range(N):
    per_iter = per_iter + _sin(x) + _cos(x)
    x = x + STEP
expected = ITERS * per_iter
diff = acc - expected
assert math.isclose(acc, expected, rel_tol=1e-9), f"checksum mismatch: {acc} vs {expected} (diff={diff})"
