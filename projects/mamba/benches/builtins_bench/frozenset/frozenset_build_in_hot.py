"""frozenset construction + membership perf bench.

End-user scenario: `frozenset(items)` then `x in fs` inside a tight
loop, the canonical immutable-set primitive that backs every cached
membership-lookup index / readonly tag set / hashable-dict-key set /
permission-pool fast check. CPython routes through PyFrozenSet_New
(C-level) + PySet_Contains; mamba's frozenset should hit a native
impl through its typed bridge.

Bounded context (DDD): builtins_bench/frozenset.

Tier: compute (with allocation pressure for the new frozenset).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `frozenset` is a builtin; no module-attr hoisting needed.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

ITEMS = list(range(0, 100, 2))
PROBES = list(range(0, 100))
# Mamba ~30x slower; cap ITERS to keep wall under ~2.5s.
ITERS = 15000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    fs = frozenset(ITEMS)
    hit = 0
    for p in PROBES:
        if p in fs:
            hit = hit + 1
    acc = acc + hit
_t1 = time.perf_counter()

print("frozenset_build_in_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref = frozenset(ITEMS)
per_iter = 0
for p in PROBES:
    if p in ref:
        per_iter = per_iter + 1
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
