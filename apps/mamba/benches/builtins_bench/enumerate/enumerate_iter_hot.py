"""enumerate() iteration — builtin perf bench.

End-user scenario: `for i, x in enumerate(xs):` over a moderately
large list, the canonical index-and-value iteration idiom that
shows up in every parser / lexer / token-emitter inner loop.
Mamba's enumerate lowers to a paired native counter + iter; CPython
dispatches FOR_ITER + UNPACK_SEQUENCE per step.

Bounded context (DDD): builtins_bench/enumerate.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
xs = list(range(N))
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for i, x in enumerate(xs):
        acc = acc + i + x
_t1 = time.perf_counter()

print("enumerate_iter_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(i for i in 0..N) + sum(xs) = 2 * (N*(N-1)//2).
expected = ITERS * 2 * (N * (N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
