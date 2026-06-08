"""itertools.groupby — consecutive-key grouping perf bench.

End-user scenario: `for k, grp in groupby(seq)` inside a tight
loop, the canonical run-length / sort-then-group primitive that
backs every event-stream coalescer / time-bucket aggregator /
log-line de-duper / segment-by-state report. CPython routes
through C-level _itertools.groupby; mamba's itertools.groupby
should hit the same algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/itertools.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `groupby` to a local.
"""

import itertools
import sys
import time

_groupby = itertools.groupby

# Pre-sorted by repeating runs to ensure non-trivial groups.
SEQ = []
for k in range(50):
    for _ in range(20):
        SEQ.append(k)
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = 0
    for _k, grp in _groupby(SEQ):
        n = 0
        for _v in grp:
            n = n + 1 + (_v & 0)
        s = s + n + (_k & 0)
    total = total + s
_t1 = time.perf_counter()

print("groupby_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Each iter sums all elements processed = len(SEQ) = 50 * 20 = 1000
expected = ITERS * len(SEQ)
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
