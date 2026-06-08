"""PEP 3132 extended iterable unpacking — perf bench.

End-user scenario: `head, *rest = xs` in a tight loop, the
canonical destructure-and-keep-tail primitive used by every
arg-shuffle / header-strip / linked-list-head walk. CPython
compiles to UNPACK_EX with a per-call list materialization for
the *rest; mamba lowers typed *-unpacks to a slice into the
source list when the source layout is known.

Bounded context (DDD): pep_bench/pep3132_star_unpack.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

The tail length stays bounded (3 elements) so we measure
unpack-machinery overhead, not heap-bound list copy cost.

ITERS capped at 100 (rather than 1000) because mamba allocates
a fresh list per *-unpack — at 1M unpacks the per-iter list
alloc dominates and pushes runtime past 30s. The reduced count
still stresses the same dispatch path on both runtimes.
"""

import sys
import time

N = 1000
# Use length-4 lists so head + *rest has a stable 3-elem tail.
xs_list = [[i, i + 1, i + 2, i + 3] for i in range(N)]
ITERS = 100

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for xs in xs_list:
        head, *rest = xs
        acc = acc + head + len(rest)
_t1 = time.perf_counter()

print("star_unpack_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(head for xs in xs_list) + N * len(rest)
# = sum(0..N-1) + N*3 = N*(N-1)//2 + 3*N.
expected = ITERS * (N * (N - 1) // 2 + 3 * N)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
