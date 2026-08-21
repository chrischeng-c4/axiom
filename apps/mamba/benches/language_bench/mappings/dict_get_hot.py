"""Dict get hot-loop bench — language-core mapping perf.

End-user scenario: tight loop over `d[k]` reads, the canonical
shape of memoised dispatch, attribute caches, and config lookups.
Mamba's MbObject::Dict wraps a HashMap behind an RwLock; CPython's
PyDict uses an open-addressing probe with cached hashes.

Bounded context (DDD): language_bench/mappings — first member of
the language-core mapping perf suite.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

Pre-built keys list avoids per-iter string interning differences
between runtimes from polluting the measurement.
"""

import sys
import time

N = 100
ITERS = 10_000  # total lookups: N * ITERS = 1M
d = {f"k{i}": i for i in range(N)}
keys = list(d.keys())

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for k in keys:
        acc = acc + d[k]
_t1 = time.perf_counter()

print("dict_get_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * (N * (N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
