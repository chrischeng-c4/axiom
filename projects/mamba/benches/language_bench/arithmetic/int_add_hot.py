"""Integer addition hot-loop bench — language-core arithmetic perf.

End-user scenario: tight accumulator loop over int + int, the most
universal primitive in numeric code (counters, checksums, histogram
bins, etc.). Mamba's force-typed JIT lowers this to a single native
ADD instruction per iteration; CPython 3.12 has to box/unbox each
operand and dispatch through PyNumber_Add → long_add.

Bounded context (DDD): language_bench/arithmetic — first member of
the language-core perf suite, separate from the existing flat
correctness fixtures under conformance/language/.

Tier: compute (per #1265 framing — mamba should beat CPython ≥10×).

#2105 avoidance: the post-loop print of `acc` happens BEFORE the
INTERNAL_TIME_NS marker, so the JIT cannot dead-code-eliminate the
hot loop based on post-marker side-effect absence.

Boxed-vs-accumulator equality workaround: per
[[project_mamba_boxed_accumulator_int_equality_bug]], a value built
via `+=` in a hot loop fails `==` against the same numeric value
built via `*`, even though both are <class 'int'>. The checksum
below uses subtraction so any divergence still surfaces but the
comparison itself sidesteps the boxed-int equality bug.
"""

import sys
import time

ITERS = 1_000_000

acc = 0
_t0 = time.perf_counter()
for i in range(ITERS):
    acc = acc + i
_t1 = time.perf_counter()

print("int_add_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Deterministic checksum — sum(0..ITERS-1) = ITERS*(ITERS-1)//2.
expected = ITERS * (ITERS - 1) // 2
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
