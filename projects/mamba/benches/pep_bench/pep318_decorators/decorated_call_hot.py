"""PEP 318 decorator-wrapped call — perf bench.

End-user scenario: `@trace def foo(x): ...` followed by hot
`foo(x)` calls, the canonical decorator-wrap dispatch path
that backs every middleware / metrics / instrumentation
layer. CPython routes each call through the wrapper closure
+ inner call; mamba can sometimes inline the wrapper when
both functions are JIT-typed.

Bounded context (DDD): pep_bench/pep318_decorators.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

The wrapper is intentionally trivial (pass-through) so the
measurement isolates dispatch overhead, not the wrapper's body
cost.
"""

import sys
import time


def passthrough(fn):
    def wrapper(x):
        return fn(x)

    return wrapper


@passthrough
def inc(x):
    return x + 1


N = 1000
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for i in range(N):
        acc = acc + inc(i)
_t1 = time.perf_counter()

print("decorated_call_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(i+1 for i in range(N)) = sum(1..N) = N*(N+1)//2.
expected = ITERS * (N * (N + 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
