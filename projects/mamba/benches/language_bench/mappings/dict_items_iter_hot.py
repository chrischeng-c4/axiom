"""dict.items() iteration — kv-view walk perf bench.

End-user scenario: `for k, v in d.items():` over a moderately-sized
dict inside a tight loop, the canonical kv-pair walk that backs every
config flatten / metric labels enumerate / form-field serialise /
inverted-index emit. CPython routes through dict_items_iter (C-level
producing tuple-pair per step); mamba's dict view should hit a
native impl through its typed bridge.

Bounded context (DDD): language_bench/mappings.

Tier: compute (with tuple-pair production per element).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; dict.items is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

D = {"k" + str(i): i for i in range(100)}
ITERS = 10000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for k, v in D.items():
        s = s + v
    acc = acc + s
_t1 = time.perf_counter()

print("dict_items_iter_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = sum(D.values())
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
