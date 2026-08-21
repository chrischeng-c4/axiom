"""functools.reduce — left-fold perf bench.

End-user scenario: `reduce(op, seq, init)` inside a tight loop, the
canonical left-fold primitive that backs every running-total /
config-merge chain / list-flatten / pipeline-compose. CPython routes
through C-level _functools.reduce; mamba's functools.reduce should
hit the same algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/functools.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `reduce` to a local.
"""

import functools
import operator
import sys
import time

_reduce = functools.reduce
_add = operator.add

N = 1000
xs = list(range(N))
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    total = total + _reduce(_add, xs, 0)
_t1 = time.perf_counter()

print("reduce_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref = sum(xs)
expected = ITERS * ref
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
