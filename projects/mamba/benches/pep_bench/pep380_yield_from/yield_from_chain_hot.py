"""PEP 380 `yield from` — generator delegation perf bench.

End-user scenario: `yield from inner()` inside a generator, the
canonical sub-iterator delegation primitive used by every
iterator-tree walk (composed pipelines, recursive flatten,
tree-traversal). CPython routes through GET_YIELD_FROM_ITER +
SEND in a tight loop; mamba's yield-from lowers to a direct
inner-generator-state pump.

Bounded context (DDD): pep_bench/pep380_yield_from.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
ITERS = 1000


def inner(n):
    for i in range(n):
        yield i


def outer(n):
    yield from inner(n)


acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in outer(N):
        acc = acc + x
_t1 = time.perf_counter()

print("yield_from_chain_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per outer iter: sum(range(N)) = N*(N-1)//2.
expected = ITERS * (N * (N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
