"""str(int) conversion — builtin perf bench.

End-user scenario: `str(x)` inside a tight loop, the canonical
int-to-decimal-string idiom that backs every log line / json
emit / template render. CPython routes through long_to_decimal_string
with a precomputed digit table; mamba's mb_str on the small-int
path lowers to a native itoa.

Bounded context (DDD): builtins_bench/str_conv — groups the
parse/format string conversion primitives.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.

The accumulator sums len() of each result to keep the per-iter
body cheap and to avoid measuring string concatenation cost.
"""

import sys
import time

N = 1000
xs = [(i * 1103515245 + 12345) & 0xFFFFFF for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        total = total + len(str(x))
_t1 = time.perf_counter()

print("str_of_int_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref_per_iter = 0
for x in xs:
    ref_per_iter = ref_per_iter + len(str(x))
expected = ITERS * ref_per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
