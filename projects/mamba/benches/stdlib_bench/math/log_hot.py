"""math.log — scalar natural-log perf bench.

End-user scenario: `math.log(x)` inside a tight loop, the canonical
log-scale primitive that backs every entropy / cross-entropy /
information-gain / log-scale-axis / softmax-stable trick. CPython
routes through C-level math.log (libm); mamba's math should hit a
native impl through its typed bridge.

Bounded context (DDD): stdlib_bench/math.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `log` to a local.
"""

import math
import sys
import time

_log = math.log

N = 10000
xs = [float(i + 1) for i in range(N)]
ITERS = 100

acc = 0.0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = 0.0
    for x in xs:
        s = s + _log(x)
    acc = acc + s
_t1 = time.perf_counter()

print("log_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref = 0.0
for x in xs:
    ref = ref + _log(x)
expected = ITERS * ref
assert math.isclose(acc, expected, rel_tol=1e-9, abs_tol=1e-3), (
    f"checksum mismatch: {acc} vs {expected}"
)
