"""Attribute write hot-loop bench — language-core class perf.

End-user scenario: tight loop over `obj.x = v` writes, the foundation
of every accumulator-object pattern / stateful aggregator / per-frame
animation tweener / streaming-state mutator. Mamba's MbObject::Instance
stores fields in an RwLock<HashMap<String, MbValue>>; writes take a
WRITE-lock + HashMap insert + SipHash on the field name. CPython's
PyObject uses an inline __dict__ insert with cached string interning,
and slot-class layouts when available.

Distinct from `attr_read_hot.py` (read path: reader-lock + probe).
Write path exercises the much hotter lock-contention + insert-with-
possible-rehash code.

Bounded context (DDD): language_bench/classes.

Tier: compute (with mutation + per-iter rehash risk).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: instance attribute assignment is syntax, not a method call —
no hoisting concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time


class Point:
    def __init__(self):
        self.x = 0
        self.y = 0
        self.z = 0


p = Point()
N = 200
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    for i in range(N):
        p.x = i
        p.y = i + 1
        p.z = i + 2
    acc = acc + p.x + p.y + p.z
_t1 = time.perf_counter()

print("attr_write_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Final values after each outer iter are always (N-1, N, N+1) → sum 3*N
per_iter = (N - 1) + N + (N + 1)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
