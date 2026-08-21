"""functools.partial — curried-callable call perf bench.

End-user scenario: `partial(fn, bound_arg)(rest)` inside a tight
loop, the canonical curried-callable primitive that backs every
pre-bound logger handler / pre-bound config-aware function /
per-request middleware / event-bus subscriber. CPython routes
through C-level _functools.partial; mamba's functools.partial
should hit the same algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/functools.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `partial` to a local.
"""

import functools
import sys
import time

_partial = functools.partial


def add3(a, b, c):
    return a + b + c


bound = _partial(add3, 10, 20)

N = 1000
# Mamba ~70x slower on this path; cap ITERS to keep wall under ~5s.
ITERS = 300

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = 0
    for i in range(N):
        s = s + bound(i)
    total = total + s
_t1 = time.perf_counter()

print("partial_call_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# bound(i) = 10 + 20 + i = 30 + i; sum_i = N*30 + N*(N-1)/2
ref = N * 30 + N * (N - 1) // 2
expected = ITERS * ref
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
