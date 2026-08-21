"""isinstance() type-guard — builtin perf bench.

End-user scenario: `isinstance(x, int)` inside a tight loop, the
canonical runtime type-guard that backs every duck-typed
function entry / dispatch table / serialization stage. CPython
routes through PyObject_IsInstance + tp_check; mamba's
mb_isinstance lowers to a tag-bit compare on the small-type
fast path.

Bounded context (DDD): builtins_bench/isinstance.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
xs = list(range(N))
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        if isinstance(x, int):
            acc = acc + 1
_t1 = time.perf_counter()

print("isinstance_int_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * N
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
