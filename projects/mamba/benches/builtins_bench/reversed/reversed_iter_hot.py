"""reversed() iteration — builtin perf bench.

End-user scenario: `for x in reversed(xs):` summing back-to-front,
the canonical reverse-traversal idiom that backs every
last-write-wins / latest-N / undo-stack walk. CPython routes
through list_reversediter; mamba's mb_reversed lowers to a
backward i64 counter when the JIT proves the source is a flat
list.

Bounded context (DDD): builtins_bench/reversed.

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
    for x in reversed(xs):
        acc = acc + x
_t1 = time.perf_counter()

print("reversed_iter_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(range(N)) = N*(N-1)//2.
expected = ITERS * (N * (N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
