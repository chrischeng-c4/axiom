"""tuple.count — element-occurrence scan perf bench.

End-user scenario: `t.count(needle)` inside a tight loop, the canonical
frozen-sequence occurrence-count primitive that backs every histogram
key tally / immutable-config flag tallying / dice-result counting /
enum-membership multiplicity check. CPython routes through
tuple_count_impl (C-level PyObject_RichCompareBool loop); mamba's
tuple should hit a native impl through its typed bridge.

Bounded context (DDD): language_bench/sequences.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; tuple.count is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

T = tuple([i % 5 for i in range(300)])
NEEDLE = 2
ITERS = 50000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    acc = acc + T.count(NEEDLE)
_t1 = time.perf_counter()

print("tuple_count_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * T.count(NEEDLE)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
