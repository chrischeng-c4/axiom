"""collections.defaultdict — group-by-key append perf bench.

End-user scenario: `dd[k].append(v)` inside a tight loop, the
canonical group-by primitive that backs every bucket-by-tenant /
partition-by-hour / fan-out-by-shard / index-by-prefix pipeline.
CPython routes through C-level _collections.defaultdict (dict +
factory hook); mamba's collections.defaultdict should hit a native
impl through its typed bridge.

Bounded context (DDD): stdlib_bench/collections.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `defaultdict` to a local.
"""

import collections
import sys
import time

_defaultdict = collections.defaultdict

N = 500
items = [(f"bucket-{i % 10}", i) for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    dd = _defaultdict(list)
    for k, v in items:
        dd[k].append(v)
    s = 0
    for k in dd:
        s = s + len(dd[k])
    total = total + s
_t1 = time.perf_counter()

print("defaultdict_groupby_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Each iter appends N items split across 10 buckets; sum of lens = N.
expected = ITERS * N
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
