"""int(str) parse — builtin perf bench.

End-user scenario: `int(s)` inside a tight loop, the canonical
decimal-string-to-int parse that backs every csv ingest /
header parse / json decimal load. CPython routes through
PyLong_FromString with a per-digit accumulator; mamba's
mb_int_from_str lowers to a native atol when the source is a
contiguous ASCII string.

Bounded context (DDD): builtins_bench/str_conv.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

The accumulator sums each parsed int to keep the per-iter body
balanced against the parse cost.
"""

import sys
import time

N = 1000
ss = [str((i * 1103515245 + 12345) & 0xFFFFFF) for i in range(N)]
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for s in ss:
        acc = acc + int(s)
_t1 = time.perf_counter()

print("int_of_str_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref_per_iter = 0
for s in ss:
    ref_per_iter = ref_per_iter + int(s)
expected = ITERS * ref_per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
