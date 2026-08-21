"""sorted() on int list — builtin perf bench.

End-user scenario: `sorted(xs)` on a moderately-large unsorted
int list, repeated. Foundation of every leaderboard / percentile /
top-k operation. CPython uses Timsort with a special long-int
fast path; mamba's mb_sorted routes through Rust's stable sort.

Bounded context (DDD): builtins_bench/sorted.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.

Workaround: mamba's force-typed system can't compute
`(N * (N-1) // 2) * ITERS` if N comes from a runtime call like
`len(xs)`. Pre-bind N as a literal.
"""

import sys
import time

# Pseudo-random but deterministic seed so both runtimes see identical input.
N = 500
xs_base = [(i * 1103515245 + 12345) & 0xFFFF for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    ys = sorted(xs_base)
    total = total + len(ys)
_t1 = time.perf_counter()

print("sort_int_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * N
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
