"""zip() two-list iteration — builtin perf bench.

End-user scenario: `for a, b in zip(xs, ys):` over equal-length
lists, the canonical parallel-iteration idiom (matching pairs,
dot product, column-wise apply).

Bounded context (DDD): builtins_bench/zip.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
xs = list(range(N))
ys = list(range(N, 2 * N))
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for a, b in zip(xs, ys):
        acc = acc + a + b
_t1 = time.perf_counter()

print("zip_pair_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(xs) + sum(ys) = sum(range(2N)) = (2N)*(2N-1)//2.
expected = ITERS * ((2 * N) * (2 * N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
