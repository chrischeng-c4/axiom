"""collections.Counter.most_common — top-N frequency perf bench.

End-user scenario: `Counter(seq).most_common(k)` inside a tight loop,
the canonical top-N frequency primitive that backs every word-cloud /
trending-tag / hot-key telemetry / log-noise reducer. CPython routes
through C-level _collections.Counter + heapq.nlargest; mamba's
collections.Counter should hit the same algorithm through its typed
bridge.

Bounded context (DDD): stdlib_bench/collections.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `Counter` to a local.
"""

import collections
import sys
import time

_Counter = collections.Counter

# 200-char sample with skewed letter freq so most_common is meaningful.
SAMPLE = ("the quick brown fox jumps over the lazy dog " * 5).strip()
ITERS = 5000
TOPK = 5

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    c = _Counter(SAMPLE)
    top = c.most_common(TOPK)
    total = total + top[0][1]
_t1 = time.perf_counter()

print("counter_most_common_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Top char in SAMPLE is space ' ' (count = 44 in the joined string).
ref = _Counter(SAMPLE).most_common(TOPK)[0][1]
expected = ITERS * ref
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
