"""dict.pop — remove-with-default perf bench.

End-user scenario: `d.pop(k, default)` inside a tight loop, the canonical
take-and-drop primitive that backs every cache-evict-and-return / LRU
demotion / kwargs consumption / pending-task dequeue. CPython routes
through dict_pop_default_impl (C-level); mamba's dict should hit a
native impl through its typed bridge.

Bounded context (DDD): language_bench/mappings.

Tier: compute (with dict mutation pressure).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; dict.pop is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

N = 50
KEYS = ["k" + str(i) for i in range(N)]
# Mamba ~110x slower; cap ITERS to keep wall under ~2.5s.
ITERS = 3000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    d = {k: i for i, k in enumerate(KEYS)}
    s = 0
    for k in KEYS:
        s = s + d.pop(k, -1)
    acc = acc + s
_t1 = time.perf_counter()

print("dict_pop_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = sum(range(N))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
