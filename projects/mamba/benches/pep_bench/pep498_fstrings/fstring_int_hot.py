"""PEP 498 f-string formatting — perf bench.

End-user scenario: `f"{x}"` inside a tight loop, the canonical
zero-arg formatted-literal rendering that backs every modern
log emit / template line / debug print. CPython compiles
f-strings to BUILD_STRING + FORMAT_VALUE; mamba lowers them to
a direct concat over precomputed segments.

Bounded context (DDD): pep_bench/pep498_fstrings.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.

The accumulator sums len() of each rendered string to keep the
per-iter body cheap and to bound the FP-free integer checksum.
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
        s = f"{x}"
        total = total + len(s)
_t1 = time.perf_counter()

print("fstring_int_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref_per_iter = 0
for x in xs:
    s = f"{x}"
    ref_per_iter = ref_per_iter + len(s)
expected = ITERS * ref_per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
