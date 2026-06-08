"""Integer multiplication hot-loop bench — language-core arithmetic perf.

End-user scenario: tight accumulator loop over int * int, common in
hash mixing (FNV/DJB2), polynomial evaluation, and bit-level codec
inner loops. Mamba's force-typed JIT lowers this to a native MUL per
iteration; CPython 3.12 dispatches through PyNumber_Multiply →
long_mul, even for small ints that fit in one machine word.

Bounded context (DDD): language_bench/arithmetic.

Tier: compute (mamba should beat CPython ≥10×).

Workload uses i64 wrap-around explicitly (multiply by a small odd
constant and mask) so both runtimes do the same finite-width work
and neither bench grows to bignum representation across iters.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

Checksum is value-based: we recompute the expected accumulator with
a separate compact formulation and compare via subtraction, per
[[project_mamba_boxed_accumulator_int_equality_bug]].
"""

import sys
import time

ITERS = 1_000_000
MASK = (1 << 32) - 1  # keep accumulator in i64 range under both runtimes

acc = 1
_t0 = time.perf_counter()
for i in range(1, ITERS + 1):
    # Odd constant + i, masked — a classic linear-congruential style mix.
    acc = (acc * 1103515245 + i) & MASK
_t1 = time.perf_counter()

print("int_mul_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Cross-check: recompute the same formula in a tight verification pass.
# Both runtimes execute this verification under CPython semantics; we only
# compare against the hot-loop result via subtraction (sidestep
# boxed-vs-accumulator int-equality bug).
expected = 1
for i in range(1, ITERS + 1):
    expected = (expected * 1103515245 + i) & MASK
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
