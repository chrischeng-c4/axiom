"""itertools.chain — two-list concat iter perf bench.

End-user scenario: `for x in chain(xs, ys):` summing across two
equal-length lists, the canonical lazy-concat traversal that
backs every result-set merge / multi-source aggregator. CPython
routes through chain_next + per-source advance; mamba's
itertools.chain lowers to a fused dual-source iter that the JIT
can keep in registers.

Bounded context (DDD): stdlib_bench/itertools.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `chain` to a local before the hot loop to avoid
module-attribute lookup churn.
"""

import sys
import time
from itertools import chain

# Hoist to local — #2097 module-attr-lookup regression workaround.
_chain = chain

N = 1000
xs = list(range(N))
ys = list(range(N, 2 * N))
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in _chain(xs, ys):
        acc = acc + x
_t1 = time.perf_counter()

print("chain_two_lists_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(range(2N)) = (2N)*(2N-1)//2.
expected = ITERS * ((2 * N) * (2 * N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
