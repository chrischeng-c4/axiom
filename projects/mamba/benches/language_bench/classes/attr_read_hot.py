"""Attribute read hot-loop bench — language-core class perf.

End-user scenario: tight loop over `obj.x` reads, the foundation of
every OOP-heavy workload (dataclass field access, accumulator
objects, struct-of-arrays views). Mamba's MbObject::Instance stores
fields in an RwLock<HashMap<String, MbValue>>; reads take a
read-lock + HashMap probe + SipHash. CPython's PyObject uses an
inline __dict__ probe with cached string interning.

Bounded context (DDD): language_bench/classes — first member of
the language-core class perf suite.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time


class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y


ITERS = 100_000
p = Point(3, 5)

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = acc + p.x + p.y
_t1 = time.perf_counter()

print("attr_read_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Literal sum — mamba's force-typed system treats p.x as a generic MbValue
# at top level, so PER_ITER_SUM is kept in sync with the Point ctor args.
PER_ITER_SUM = 3 + 5
expected = ITERS * PER_ITER_SUM
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
