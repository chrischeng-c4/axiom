"""divmod() on ints — builtin perf bench.

End-user scenario: `divmod(n, d)` inside a tight loop, the
canonical paired quotient+remainder primitive used by every
base-conversion / time-bucket / pagination split. CPython
fuses long_divmod into a single PyLong op; mamba's mb_divmod
lowers to a paired native sdiv + msub.

Bounded context (DDD): builtins_bench/divmod.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

Mamba force-typed: tuple-unpacking the result then summing both
halves stresses the dispatch path most users actually trigger
(`q, r = divmod(n, d)`).
"""

import sys
import time

N = 1000
xs = [(i * 7 + 13) for i in range(N)]
D = 17
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        q, r = divmod(x, D)
        acc = acc + q + r
_t1 = time.perf_counter()

print("divmod_int_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref_per_iter = 0
for x in xs:
    q, r = divmod(x, D)
    ref_per_iter = ref_per_iter + q + r
expected = ITERS * ref_per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
