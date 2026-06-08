"""Scalar bulk Fraction arithmetic (Task #45, Wave-3 ship #2).

Predicted regime per scout doc: compute (GCD reduction dominates over
allocation). Each iteration constructs a fresh Fraction handle via
the constructor + chains add / sub / mul.

**Operator-overloading carve-out** (documented in `fractions_mod.rs`):
mamba's JIT lowers `a + b` to a native i64 add because the handle IS
an int — never reaching `class.rs::mb_call_method.__add__`. The bench
therefore uses **module-level functions** (`fractions.fraction_add`,
`fractions.fraction_mul`, etc.) which CPython's `fractions` module
does NOT expose. To make the bench cross-runtime comparable, this
fixture monkey-patches the missing module-level fns onto
`fractions` at the top of the script when running under CPython
(detected by `sys.implementation.name`). Both runtimes then see the
same names and call shapes.

NB on hasattr-conditional bindings: an earlier draft used
`if hasattr(fractions, "fraction_add"): X = fractions.X else: def X(...)`
to pick the binding. That pattern triggers a JIT type-confusion under
mamba (the conditional makes the JIT model `fraction_add` as both a
module-fn-pointer AND an `MbObject::Function`, and the call result is
dropped — same fingerprint as
`project_mamba_jit_drops_branches_after_stdlib_call`). Workaround:
do the monkey-patch BEFORE any subsequent code touches the symbols
so the binding is unconditional at the per-call site.

The Fraction handle is an int — `Fraction(num, den)` returns an i64 ID
indexing a thread_local table. Arith dispatches to free fns and the
result is a fresh i64 handle. **No tuple allocation on the arithmetic
hot path** → not subject to the #2128 tuple-alloc carve-out that
penalised colorsys. The only carve-out tuple path is
`fractions.fraction_divmod`, which is NOT exercised by this bench.
Bench expectation: compute-leaning, target ≥0.7× internal vs CPython
per scout doc estimate.

Hoist convention (#2097): bind module-level callables BEFORE the
loop so each iter is a direct call, not a per-iter module-attr lookup.

# tier: compute
"""

import fractions
import sys

# Unconditional CPython-side adapter: install module-level arith fns
# so both runtimes export the same surface. On mamba these names
# already exist as native dispatchers and the assignment is a no-op
# rebind. The conditional is on `sys.implementation.name` so the JIT
# sees only ONE binding shape per name at the call sites below.
if sys.implementation.name == "cpython":
    fractions.fraction_add = lambda a, b: a + b
    fractions.fraction_sub = lambda a, b: a - b
    fractions.fraction_mul = lambda a, b: a * b
    fractions.fraction_numerator = lambda a: a.numerator
    fractions.fraction_denominator = lambda a: a.denominator

Fraction = fractions.Fraction
fraction_add = fractions.fraction_add
fraction_sub = fractions.fraction_sub
fraction_mul = fractions.fraction_mul
fraction_numerator = fractions.fraction_numerator
fraction_denominator = fractions.fraction_denominator

ITERS = 100_000

acc_num = 0
acc_den = 0
for i in range(ITERS):
    a = Fraction((i & 31) + 1, ((i >> 3) & 15) + 1)
    b = Fraction(((i >> 1) & 7) + 1, ((i >> 5) & 3) + 1)
    s = fraction_add(a, b)
    d = fraction_sub(s, a)
    p = fraction_mul(d, b)
    acc_num += fraction_numerator(p)
    acc_den += fraction_denominator(p)
print("fractions_arith:", acc_num, acc_den)
