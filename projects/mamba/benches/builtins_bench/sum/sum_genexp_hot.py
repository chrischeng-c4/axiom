"""sum(genexp) — fold-over-generator builtin perf bench.

End-user scenario: `sum(x * x for x in xs)` inside a tight loop, the
canonical map-then-fold primitive that backs every total-cost compute /
weighted-sum metric / variance numerator / dot-product fragment.
CPython routes through builtin_sum_impl + per-step PyObject_Next on
the genexp; mamba's sum should hit a native impl + JIT-fused genexp
body through its typed bridge.

Bounded context (DDD): builtins_bench/sum.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `sum` is a builtin; no module-attr hoisting needed.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

N = 500
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    acc = acc + sum(x * x for x in range(N))
_t1 = time.perf_counter()

print("sum_genexp_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = sum(x * x for x in range(N))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
