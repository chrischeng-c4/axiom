"""int.bit_length + int.bit_count — int-bit-ops perf bench.

End-user scenario: `n.bit_length()` and `n.bit_count()` inside a tight
loop, the canonical popcount/leading-bit primitives that back every
hamming-distance compute / bitmap density gauge / hash bucket
allocation sizer / variable-length integer encoder. CPython routes
through C-level _PyLong_NumBits / popcount intrinsic; mamba's int
should hit native ops through its typed bridge.

Bounded context (DDD): language_bench/integers.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: int methods are dispatched on the value; no module attrs.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

# Mamba per-int-method dispatch is ~145x slower than CPython's C-level
# popcount intrinsic. ITERS capped at 500 to keep mamba wall under ~3s.
N = 500
ITERS = 500

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for v in range(N):
        s = s + v.bit_length() + v.bit_count()
    acc = acc + s
_t1 = time.perf_counter()

print("bit_length_count_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for v in range(N):
    per_iter = per_iter + v.bit_length() + v.bit_count()
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
