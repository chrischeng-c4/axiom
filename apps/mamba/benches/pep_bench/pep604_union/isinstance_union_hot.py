"""PEP 604 union syntax — isinstance(x, A | B) perf bench.

End-user scenario: `isinstance(x, int | float)` inside a tight
loop, the canonical multi-type runtime guard introduced as a
PEP 604 ergonomic. CPython routes through PyObject_IsInstance
+ types.UnionType match; mamba's mb_isinstance with a Union
RHS lowers to a flat tag-bit OR-chain.

Bounded context (DDD): pep_bench/pep604_union.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
# Alternating int / float so both legs of the union actually fire.
xs = []
for i in range(N):
    if (i & 1) == 0:
        xs.append(i)
    else:
        xs.append(float(i))
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        if isinstance(x, int | float):
            acc = acc + 1
_t1 = time.perf_counter()

print("isinstance_union_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Every element matches the union, so every iter increments acc.
expected = ITERS * N
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
