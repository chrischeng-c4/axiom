"""repr() — object-to-debug-string builtin perf bench.

End-user scenario: `repr(obj)` over heterogenous objects inside a tight
loop, the canonical debug-format primitive that backs every log dump
helper / exception message format / test-assertion failure diff /
REPL echo. CPython routes through tp_repr (C-level per-type slot);
mamba's builtins should hit native repr slots through its typed
bridge.

Bounded context (DDD): builtins_bench/repr.

Tier: compute (with new-string allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `repr` is a builtin; no module-attr hoisting needed.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

OBJS = [42, -7, "hello", "world", 3.14, [1, 2, 3], (4, 5, 6), True, None, ""]
ITERS = 50000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for o in OBJS:
        s = s + len(repr(o))
    acc = acc + s
_t1 = time.perf_counter()

print("repr_mixed_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for o in OBJS:
    per_iter = per_iter + len(repr(o))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
