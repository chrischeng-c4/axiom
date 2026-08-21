"""len() on list — builtin perf bench.

End-user scenario: tight loop over `len(xs)` for the same list,
the canonical guard-condition shape in every iteration / dispatch /
batch-size-check codepath. Mamba's mb_len reads Vec::len via the
List's RwLock; CPython's PyList_Size is an inline pointer-deref +
field read.

Bounded context (DDD): builtins_bench/len — first member of the
builtins perf suite (directive #4).

Tier: startup (per-call cost is ~constant, total wall is dominated
by Python startup amortisation; mamba's edge here is reading
Vec::len without the boxed-int wrapper allocation that CPython's
PyLong_FromSsize_t pays).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

xs = list(range(1000))
ITERS = 1_000_000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = acc + len(xs)
_t1 = time.perf_counter()

print("list_len_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * 1000
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
