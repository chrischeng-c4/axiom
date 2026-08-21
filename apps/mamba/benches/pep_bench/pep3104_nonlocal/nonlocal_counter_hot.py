"""PEP 3104 `nonlocal` — closure write-through perf bench.

End-user scenario: a counter closure that captures + mutates an
outer-scope int via `nonlocal`, the canonical lightweight
mutable-state-without-class pattern (id factories, retry
counters, sliding accumulators). CPython routes through
LOAD_DEREF + STORE_DEREF over a cell; mamba lowers a typed
nonlocal int to a captured slot the JIT mutates in place.

Bounded context (DDD): pep_bench/pep3104_nonlocal.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time


def make_counter():
    n = 0

    def inc():
        nonlocal n
        n = n + 1
        return n

    return inc


N = 1000
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    c = make_counter()
    for _i in range(N):
        total = total + c()
_t1 = time.perf_counter()

print("nonlocal_counter_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per outer iter: c() returns 1..N, so sum = N*(N+1)//2.
expected = ITERS * (N * (N + 1) // 2)
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
