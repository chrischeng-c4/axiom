"""Tuple unpack hot-loop bench — language-core sequence perf.

End-user scenario: tight loop over `a, b = pair` reads, the canonical
shape for multi-return ABI, swap idioms, and iter-of-pairs patterns.
Mamba's tuple unpacking lowers to direct field copies; CPython
dispatches through UNPACK_SEQUENCE bytecode.

Bounded context (DDD): language_bench/sequences.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

ITERS = 1_000_000
PAIR = (3, 5)

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    a, b = PAIR
    acc = acc + a + b
_t1 = time.perf_counter()

print("tuple_unpack_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Mamba's force-typed system treats PAIR[i] as a generic MbValue, not an
# inferred int, so arithmetic on indexed tuple elements raises a static
# type error. Use a literal sum (matches PAIR contents above) so both
# runtimes see the same checksum without invoking tuple-index arithmetic
# outside the hot loop. The hot loop itself binds via unpack syntax which
# DOES get type-narrowed by mamba's lowering.
PER_ITER_SUM = 3 + 5  # keep in sync with PAIR literal above
expected = ITERS * PER_ITER_SUM
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
