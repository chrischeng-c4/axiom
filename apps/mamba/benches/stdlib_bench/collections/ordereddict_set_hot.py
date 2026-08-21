"""collections.OrderedDict — insertion-order dict set perf bench.

End-user scenario: `od[k] = v` inside a tight loop, the canonical
insertion-order primitive that backs every LRU cache / move-to-end
recency tracker / config-merge with order. CPython routes through
_collections.OrderedDict (C-level doubly-linked-list + dict); mamba's
collections.OrderedDict should hit the same native impl through its
typed bridge.

Bounded context (DDD): stdlib_bench/collections.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `OrderedDict` to a local.
"""

import collections
import sys
import time

_OD = collections.OrderedDict

N = 1000
keys = [f"k-{i}" for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    od = _OD()
    for i, k in enumerate(keys):
        od[k] = i
    total = total + len(od)
_t1 = time.perf_counter()

print("ordereddict_set_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Each iter inserts N distinct keys.
expected = ITERS * N
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
