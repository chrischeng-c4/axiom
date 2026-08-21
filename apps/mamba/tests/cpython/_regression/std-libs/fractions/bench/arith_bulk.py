"""Scalar bulk Fraction arithmetic (Task #45, Wave-3 ship #2).

Predicted regime per scout doc: compute (GCD reduction dominates over
allocation). Each iteration constructs a fresh Fraction handle via
the constructor + chains add / sub / mul.

**Operator-overloading carve-out RESOLVED (#961/#2129):** mamba's JIT
used to lower `a + b` to a native i64 add whenever both operands were
statically `Ty::Int` — which a `fractions.Fraction` handle is (a
NaN-boxed inline int indexing a thread_local table) — so arithmetic
never reached `class.rs::mb_call_method.__add__`, silently adding raw
handle ids instead of dispatching the dunder. This bench used to route
around the gap via module-level dispatcher functions
(`fractions.fraction_add` etc., monkey-patched onto CPython's
`fractions` for comparability). Now that `bigint_ops::mb_bigint_{add,
sub,mul}` guard their boxed/slow path with a handle-protocol check
(see the guard note on `mb_bigint_add`) before falling through to
plain BigInt-aware int math, direct operators dispatch correctly on
both runtimes and the workaround is unnecessary.

The Fraction handle is an int — `Fraction(num, den)` returns an i64 ID
indexing a thread_local table. Arith dispatches through the guarded
binop path and the result is a fresh i64 handle. **No tuple allocation
on the arithmetic hot path** → not subject to the #2128 tuple-alloc
carve-out that penalised colorsys. The only carve-out tuple path is
`fractions.fraction_divmod`, which is NOT exercised by this bench.
Bench expectation: compute-leaning, target ≥0.7× internal vs CPython
per scout doc estimate.

# tier: compute
"""

import fractions

Fraction = fractions.Fraction

ITERS = 100_000

acc_num = 0
acc_den = 0
for i in range(ITERS):
    a = Fraction((i & 31) + 1, ((i >> 3) & 15) + 1)
    b = Fraction(((i >> 1) & 7) + 1, ((i >> 5) & 3) + 1)
    s = a + b
    d = s - a
    p = d * b
    acc_num += p.numerator
    acc_den += p.denominator
print("fractions_arith:", acc_num, acc_den)
