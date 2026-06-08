# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_math_float_repr_ops"
# subject = "cpython321.test_math_float_repr_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_math_float_repr_ops.py"
# status = "filled"
# ///
"""cpython321.test_math_float_repr_ops: execute CPython 3.12 seed test_math_float_repr_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for math float-representation and
# transcendental extras not exercised by existing math fixtures.
# Surface:
#   • math.frexp(x) — split into (mantissa, exponent) such that
#     x == mantissa * 2**exponent and 0.5 <= |mantissa| < 1.0;
#   • math.ldexp(m, e) — reverse: m * 2**e;
#   • math.fsum(iter) — exact sum of floats (more accurate than
#     builtin sum);
#   • math.log(x, base) — log with explicit base;
#   • math.log1p(x) — log(1+x), accurate for small x;
#   • math.expm1(x) — exp(x)-1, accurate for small x;
#   • math.atan2(y, x) — quadrant-aware arctangent;
#   • math.pow(x, y) — float-returning power (vs operator ** which
#     can return int);
#   • hyperbolic family — sinh/cosh/tanh and inverse asinh/acosh/atanh;
#   • math.inf / math.nan / -math.inf module constants.
#
# math.isclose with abs_tol= and math.erf/erfc are deliberately NOT
# exercised here — mamba 0.3.60 disagrees on the abs_tol semantics
# and lacks the erf/erfc surface. Both move to separate seeds.
import math
_ledger: list[int] = []

# frexp — positive
_m, _e = math.frexp(8.0)
assert _m == 0.5; _ledger.append(1)
assert _e == 4; _ledger.append(1)

_m2, _e2 = math.frexp(0.5)
assert _m2 == 0.5; _ledger.append(1)
assert _e2 == 0; _ledger.append(1)

# frexp — zero
_m3, _e3 = math.frexp(0.0)
assert _m3 == 0.0; _ledger.append(1)
assert _e3 == 0; _ledger.append(1)

# frexp — negative
_m4, _e4 = math.frexp(-8.0)
assert _m4 == -0.5; _ledger.append(1)
assert _e4 == 4; _ledger.append(1)

# ldexp — inverse of frexp for the representable cases
assert math.ldexp(0.5, 4) == 8.0; _ledger.append(1)
assert math.ldexp(1.0, 0) == 1.0; _ledger.append(1)
assert math.ldexp(1.0, 10) == 1024.0; _ledger.append(1)
assert math.ldexp(0.5, -1) == 0.25; _ledger.append(1)
assert math.ldexp(-0.5, 3) == -4.0; _ledger.append(1)

# fsum — empty / exact / accurate
assert math.fsum([]) == 0.0; _ledger.append(1)
assert math.fsum([1.0, 2.0, 3.0]) == 6.0; _ledger.append(1)
assert math.fsum([1.0]) == 1.0; _ledger.append(1)
# fsum is more accurate than sum: 0.1 + 0.1 + ... summed ten times
# yields exactly 1.0 under fsum (sum would drift)
assert math.fsum([0.1] * 10) == 1.0; _ledger.append(1)

# log with explicit base
assert math.log(8, 2) == 3.0; _ledger.append(1)
assert math.log(100, 10) == 2.0; _ledger.append(1)
assert math.log(27, 3) == 3.0; _ledger.append(1)
assert math.log(1, 10) == 0.0; _ledger.append(1)

# log1p — log(1 + x), zero exact at x=0
assert math.log1p(0) == 0.0; _ledger.append(1)
assert abs(math.log1p(1) - math.log(2)) < 1e-12; _ledger.append(1)
assert abs(math.log1p(math.e - 1) - 1.0) < 1e-12; _ledger.append(1)

# expm1 — exp(x) - 1, zero exact at x=0
assert math.expm1(0) == 0.0; _ledger.append(1)
assert abs(math.expm1(1) - (math.e - 1)) < 1e-12; _ledger.append(1)

# atan2 — quadrant-aware
assert math.atan2(0, 1) == 0.0; _ledger.append(1)
assert math.atan2(1, 0) == math.pi / 2; _ledger.append(1)
assert math.atan2(-1, 0) == -math.pi / 2; _ledger.append(1)
assert math.atan2(0, -1) == math.pi; _ledger.append(1)

# math.pow — float-returning power
assert math.pow(2, 10) == 1024.0; _ledger.append(1)
assert math.pow(2, -1) == 0.5; _ledger.append(1)
assert math.pow(0, 0) == 1.0; _ledger.append(1)
assert math.pow(1, 100) == 1.0; _ledger.append(1)
assert math.pow(9.0, 0.5) == 3.0; _ledger.append(1)

# Hyperbolic at 0
assert math.sinh(0) == 0.0; _ledger.append(1)
assert math.cosh(0) == 1.0; _ledger.append(1)
assert math.tanh(0) == 0.0; _ledger.append(1)

# Inverse hyperbolic at boundary
assert math.asinh(0) == 0.0; _ledger.append(1)
assert math.acosh(1) == 0.0; _ledger.append(1)
assert math.atanh(0) == 0.0; _ledger.append(1)

# Hyperbolic identities — cosh^2 - sinh^2 == 1
_v = 1.5
assert abs(math.cosh(_v) ** 2 - math.sinh(_v) ** 2 - 1.0) < 1e-12; _ledger.append(1)

# math.inf and math.nan module constants
assert math.inf > 1e308; _ledger.append(1)
assert -math.inf < -1e308; _ledger.append(1)
assert math.nan != math.nan; _ledger.append(1)
assert math.isnan(math.nan); _ledger.append(1)
assert math.isinf(math.inf); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_math_float_repr_ops {sum(_ledger)} asserts")
