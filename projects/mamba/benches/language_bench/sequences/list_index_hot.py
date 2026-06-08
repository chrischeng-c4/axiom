"""List indexing hot-loop bench — language-core sequence perf.

End-user scenario: tight loop over `xs[i]` reads, the foundation of
every numerical / graph / DSP inner loop that hasn't moved to numpy.
Mamba's MbObject::List wraps a Vec<MbValue> behind an RwLock; index
reads take a read-lock per access. CPython's PyList stores items in
a contiguous `PyObject **` and indexes with a single MOV.

Bounded context (DDD): language_bench/sequences — first member of
the language-core sequence perf suite.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
ITERS = 1000  # total accesses: N * ITERS = 1M
xs = list(range(N))

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for i in range(N):
        acc = acc + xs[i]
_t1 = time.perf_counter()

print("list_index_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * (N * (N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
