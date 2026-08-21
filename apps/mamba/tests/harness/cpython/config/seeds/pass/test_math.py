# test_math.py — #2835 CPython math seed (executed assertions).
#
# Mamba-authored seed distilled from the math module surface. Exercises
# the load-bearing helpers downstream users actually reach for:
# constants (pi, e, tau, inf, nan); rounding (ceil/floor/trunc/fabs);
# integer arithmetic (factorial, gcd); transcendentals (sqrt, pow,
# log, exp, sin, cos); and float classification (isfinite/isnan).
# Emits the runner's positive proof-of-execution marker that
# `cpython_lib_test_runner.rs` (#2691) classifies as `AssertionPass`.
#
# Floating-point comparisons use `abs(x - expected) < tol` with tol =
# 1e-10 — the seed's stable-tolerance contract per the #2835 acceptance.
# Where the answer is exactly representable in float (sqrt(4) == 2.0,
# pow(2, 10) == 1024.0, sin(0) == 0.0), direct `==` is fine.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: math N asserts` to stdout.

import math

_ledger: list[int] = []

# 1. Module identity + public surface.
assert math.__name__ == "math", "math.__name__ must be 'math'"
_ledger.append(1)
assert hasattr(math, "pi"), "math must expose pi"
_ledger.append(1)
assert hasattr(math, "e"), "math must expose e"
_ledger.append(1)
assert hasattr(math, "sqrt"), "math must expose sqrt"
_ledger.append(1)
assert hasattr(math, "ceil"), "math must expose ceil"
_ledger.append(1)
assert hasattr(math, "floor"), "math must expose floor"
_ledger.append(1)
assert hasattr(math, "factorial"), "math must expose factorial"
_ledger.append(1)

# 2. Constants — π and e to canonical 15-decimal precision; tau == 2π.
assert abs(math.pi - 3.141592653589793) < 1e-10, "math.pi ≈ 3.141592653589793"
_ledger.append(1)
assert abs(math.e - 2.718281828459045) < 1e-10, "math.e ≈ 2.718281828459045"
_ledger.append(1)
assert abs(math.tau - 2 * math.pi) < 1e-10, "math.tau == 2 * math.pi"
_ledger.append(1)

# 3. sqrt — exact roots are exactly representable.
assert math.sqrt(4) == 2.0, "math.sqrt(4) == 2.0 (exact)"
_ledger.append(1)
assert math.sqrt(9) == 3.0, "math.sqrt(9) == 3.0 (exact)"
_ledger.append(1)
assert abs(math.sqrt(2) * math.sqrt(2) - 2) < 1e-10, "sqrt(2)² ≈ 2 (irrational, tolerant)"
_ledger.append(1)

# 4. Rounding family — return integers.
assert math.ceil(2.3) == 3, "math.ceil(2.3) == 3"
_ledger.append(1)
assert math.ceil(-2.3) == -2, "math.ceil(-2.3) == -2 (toward +inf)"
_ledger.append(1)
assert math.floor(2.7) == 2, "math.floor(2.7) == 2"
_ledger.append(1)
assert math.floor(-2.3) == -3, "math.floor(-2.3) == -3 (toward -inf)"
_ledger.append(1)
assert math.trunc(2.7) == 2, "math.trunc(2.7) == 2"
_ledger.append(1)
assert math.trunc(-2.7) == -2, "math.trunc(-2.7) == -2 (toward zero)"
_ledger.append(1)
assert math.fabs(-3.5) == 3.5, "math.fabs(-3.5) == 3.5"
_ledger.append(1)
assert math.fabs(3.5) == 3.5, "math.fabs(3.5) == 3.5"
_ledger.append(1)

# 5. Integer arithmetic.
assert math.factorial(0) == 1, "math.factorial(0) == 1"
_ledger.append(1)
assert math.factorial(5) == 120, "math.factorial(5) == 120"
_ledger.append(1)
assert math.factorial(6) == 720, "math.factorial(6) == 720"
_ledger.append(1)
assert math.gcd(12, 8) == 4, "math.gcd(12, 8) == 4"
_ledger.append(1)
assert math.gcd(17, 13) == 1, "math.gcd(17, 13) == 1 (coprime)"
_ledger.append(1)

# 6. Power / log / exp — exactly representable cases use `==`,
#    transcendentals use the tolerance bound.
assert math.pow(2, 10) == 1024.0, "math.pow(2, 10) == 1024.0 (exact)"
_ledger.append(1)
assert math.pow(3, 4) == 81.0, "math.pow(3, 4) == 81.0 (exact)"
_ledger.append(1)
assert abs(math.log(math.e) - 1.0) < 1e-10, "math.log(e) ≈ 1.0"
_ledger.append(1)
assert math.exp(0) == 1.0, "math.exp(0) == 1.0 (exact)"
_ledger.append(1)
assert abs(math.exp(1) - math.e) < 1e-10, "math.exp(1) ≈ math.e"
_ledger.append(1)

# 7. Trigonometry — anchor points.
assert math.sin(0) == 0.0, "math.sin(0) == 0.0 (exact)"
_ledger.append(1)
assert math.cos(0) == 1.0, "math.cos(0) == 1.0 (exact)"
_ledger.append(1)
assert abs(math.sin(math.pi)) < 1e-10, "math.sin(π) ≈ 0 (tolerant)"
_ledger.append(1)
assert abs(math.cos(math.pi) + 1.0) < 1e-10, "math.cos(π) ≈ -1.0 (tolerant)"
_ledger.append(1)

# 8. Float classification.
assert math.isfinite(1.0) == True, "math.isfinite(1.0) == True"
_ledger.append(1)
assert math.isfinite(math.inf) == False, "math.isfinite(inf) == False"
_ledger.append(1)
assert math.isnan(0.0) == False, "math.isnan(0.0) == False"
_ledger.append(1)
assert math.isnan(math.nan) == True, "math.isnan(nan) == True"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: math {len(_ledger)} asserts")
