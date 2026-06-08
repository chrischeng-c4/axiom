"""dict.update — bulk-merge perf bench.

End-user scenario: `target.update(delta)` inside a tight loop, the
canonical config-merge primitive that backs every settings overlay /
env-var precedence apply / request-context patch / kwargs blend.
CPython routes through dict_update_common (C-level resize-aware
merge); mamba's dict should hit a native impl through its typed
bridge.

Bounded context (DDD): language_bench/mappings.

Tier: compute (with allocation pressure on the fresh target dict).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; dict.update is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

BASE = {"k" + str(i): i for i in range(50)}
DELTA = {"k" + str(i): i * 10 for i in range(50)}
ITERS = 60000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    target = {}
    target.update(BASE)
    target.update(DELTA)
    acc = acc + len(target)
_t1 = time.perf_counter()

print("dict_update_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref_t = {}
ref_t.update(BASE)
ref_t.update(DELTA)
expected = ITERS * len(ref_t)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
