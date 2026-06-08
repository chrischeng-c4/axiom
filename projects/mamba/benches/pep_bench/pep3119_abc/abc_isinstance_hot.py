"""PEP 3119 ABC subclasshook — abc.Sized perf bench.

End-user scenario: `isinstance(x, Sized)` inside a tight loop,
the canonical structural-protocol guard backed by an ABC with
__subclasshook__ (returns True for any object with __len__).
CPython routes through ABCMeta.__instancecheck__ + cache;
mamba's mb_isinstance with an ABC RHS hits the same cache but
through its own typed path.

Bounded context (DDD): pep_bench/pep3119_abc.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time
from collections.abc import Sized

N = 1000
xs = [list(range(i & 0x0F)) for i in range(N)]
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        if isinstance(x, Sized):
            acc = acc + 1
_t1 = time.perf_counter()

print("abc_isinstance_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Every list is Sized, so every iter increments acc.
expected = ITERS * N
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
