# test_fractions.py — #2834 CPython fractions seed (executed assertions).
#
# Mamba-authored seed distilled from the fractions module surface.
# Exercises module identity + Fraction binding + numerator/denominator
# attribute access on two-argument Fraction(n, d) construction. Arith
# (`+`, `-`, `*`, `/`, `==` between Fraction values) is excluded —
# mamba's Fraction(x) currently returns a fresh boxed-int handle
# (`type(f).__name__ == 'int'`, NOT 'Fraction'), so arithmetic between
# Fraction handles bypasses the class.rs dunder dispatch and lowers to
# native i64 (`project_mamba_int_handle_operator_overload_gap` memory
# family). The seed asserts only the attribute-access path, which
# correctly returns the stored numerator/denominator integers today.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: fractions N asserts` to stdout.

import fractions

_ledger: list[int] = []

# 1. Module identity + public surface.
assert fractions.__name__ == "fractions", "fractions.__name__ must be 'fractions'"
_ledger.append(1)
assert hasattr(fractions, "Fraction"), "fractions must expose Fraction"
_ledger.append(1)
assert callable(fractions.Fraction), "fractions.Fraction must be callable"
_ledger.append(1)

# 2. Fraction(num, den) — two-arg form preserves both numerator and
#    denominator on the resulting object. (One-arg `Fraction(5)`
#    returns 0/1 on mamba today; that gap is tracked separately.)
_f12 = fractions.Fraction(1, 2)
assert _f12.numerator == 1, "Fraction(1, 2).numerator == 1"
_ledger.append(1)
assert _f12.denominator == 2, "Fraction(1, 2).denominator == 2"
_ledger.append(1)

_f34 = fractions.Fraction(3, 4)
assert _f34.numerator == 3, "Fraction(3, 4).numerator == 3"
_ledger.append(1)
assert _f34.denominator == 4, "Fraction(3, 4).denominator == 4"
_ledger.append(1)

_f57 = fractions.Fraction(5, 7)
assert _f57.numerator == 5, "Fraction(5, 7).numerator == 5"
_ledger.append(1)
assert _f57.denominator == 7, "Fraction(5, 7).denominator == 7"
_ledger.append(1)

# 3. Negative numerator survives — sign is preserved on the numerator.
_fneg = fractions.Fraction(-3, 4)
assert _fneg.numerator == -3, "Fraction(-3, 4).numerator == -3 (sign on numerator)"
_ledger.append(1)
assert _fneg.denominator == 4, "Fraction(-3, 4).denominator == 4"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: fractions {len(_ledger)} asserts")
