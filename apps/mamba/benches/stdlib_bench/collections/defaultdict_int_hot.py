"""collections.defaultdict(int) — bucket-fill perf bench.

End-user scenario: `d[k] += 1` over a defaultdict(int), the
canonical bucket-accumulator pattern that backs every word
count / event tally / category histogram. CPython's
defaultdict.__missing__ is a C tp_subscript hook; mamba's
defaultdict is a Python-level wrap with per-miss __missing__
dispatch.

Bounded context (DDD): stdlib_bench/collections.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `defaultdict` to a local before the hot loop.
"""

import sys
import time
from collections import defaultdict

_defaultdict = defaultdict

N = 1000
keys = [(i & 0x1F) for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    d = _defaultdict(int)
    for k in keys:
        d[k] = d[k] + 1
    total = total + len(d)
_t1 = time.perf_counter()

print("defaultdict_int_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Keys are 0..31 (32 distinct), so each iter d has 32 entries.
expected = ITERS * 32
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
