"""math.tan / math.exp — transcendental-pair perf bench.

End-user scenario: `math.tan(theta)` and `math.exp(rate * t)` inside a
tight loop, the canonical transcendental-fold primitive that backs
every angle-slope solver / sigmoid evaluator / Boltzmann-weight
accumulator / continuously-compounded-interest summer. CPython routes
through libm tan/exp via math_2 helpers; mamba's math should hit a
native FFI thunk through its typed bridge.

Bounded context (DDD): stdlib_bench/math.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `math.tan` / `math.exp` module attrs to locals — they
ARE module-level free fns (NOT bound methods), so the hoist is safe.
DO NOT hoist `_print = print` or any bound method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars. `import a, b` only binds `a` under mamba — use
separate `import` lines.
"""

import math
import sys
import time

_tan = math.tan
_exp = math.exp

N = 200
ITERS = 5000

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    for i in range(N):
        x = float(i) * 0.005
        s = s + _tan(x) + _exp(x)
    acc = acc + s
_t1 = time.perf_counter()

print("tan_exp_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0.0
for i in range(N):
    x = float(i) * 0.005
    per_iter = per_iter + math.tan(x) + math.exp(x)
expected = ITERS * per_iter
# FP accumulator — use isclose, not == (associativity drift).
assert math.isclose(acc, expected, rel_tol=1e-9), f"checksum drift: {acc} vs {expected}"
