"""itertools.accumulate — cumulative-sum perf bench.

End-user scenario: `accumulate(xs)` inside a tight loop, the canonical
running-total primitive that backs every cumulative-distribution build /
prefix-sum index / running-balance ledger / time-series CDF.  CPython
routes through C-level _itertools.accumulate; mamba's itertools should
hit the same algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/itertools.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `accumulate` to a local.

NOTE: do NOT use the `initial=` kwarg under mamba — it returns
None for all tail elements (separate runtime gap).
"""

import itertools
import sys
import time

_accumulate = itertools.accumulate

N = 1000
xs = list(range(N))
ITERS = 500

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    out_sum = 0
    for v in _accumulate(xs):
        out_sum = out_sum + v
    total = total + out_sum
_t1 = time.perf_counter()

print("accumulate_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for v in _accumulate(xs):
    per_iter = per_iter + v
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
