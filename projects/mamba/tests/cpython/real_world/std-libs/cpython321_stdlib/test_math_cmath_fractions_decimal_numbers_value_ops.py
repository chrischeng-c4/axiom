# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_math_cmath_fractions_decimal_numbers_value_ops"
# subject = "cpython321.test_math_cmath_fractions_decimal_numbers_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_math_cmath_fractions_decimal_numbers_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_math_cmath_fractions_decimal_numbers_value_ops: execute CPython 3.12 seed test_math_cmath_fractions_decimal_numbers_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `math` / `cmath` / `fractions` / `decimal` / `numbers` five-
# pack pinned to atomic 189: `math` (the documented full
# module-level helper hasattr surface — `pi` / `e` / `tau` /
# `inf` / `nan` / `sqrt` / `floor` / `ceil` / `trunc` / `log`
# / `log2` / `log10` / `exp` / `sin` / `cos` / `tan` / `asin`
# / `acos` / `atan` / `atan2` / `factorial` / `gcd` / `lcm`
# / `isfinite` / `isinf` / `isnan` / `fabs` / `pow` / `hypot`
# + the documented math.pi / math.e / math.tau / math.sqrt /
# math.floor / math.ceil / math.factorial / math.gcd /
# math.isfinite / math.isinf / math.isnan / math.log value
# contract), `cmath` (the documented partial module-level
# helper hasattr surface — `pi` / `e` / `sqrt` / `phase` /
# `polar` / `rect` / `exp` / `log` / `isnan` / `isinf` + the
# documented cmath.sqrt(-1) == 1j / cmath.pi / cmath.phase
# value contract), `fractions` (the documented partial
# module-level helper hasattr surface — `Fraction` + the
# documented Fraction(1, 4).numerator / .denominator
# attribute contract), `decimal` (the documented partial
# module-level helper hasattr surface — `Decimal`), and
# `numbers` (the documented full module-level helper hasattr
# surface — `Number` / `Complex` / `Real` / `Rational` /
# `Integral`).
#
# The matching subset between mamba and CPython is the full
# `math` module hasattr surface + the math.pi / e / tau /
# sqrt / floor / ceil / factorial / gcd / isfinite / isinf
# / isnan / log value layer, the partial `cmath` module
# hasattr surface (pi / e / sqrt / phase / polar / rect /
# exp / log / isnan / isinf) + the cmath.sqrt(-1) / pi /
# phase value layer, the partial `fractions` module hasattr
# surface (`Fraction` — `gcd` is not a fractions attribute
# in modern CPython, was removed) + the .numerator /
# .denominator attribute layer (the class identity layer
# DIVERGES + the Fraction + Fraction arithmetic layer
# DIVERGES), the partial `decimal` module hasattr surface
# (`Decimal` — the extended `getcontext` / `setcontext` /
# `localcontext` / `ROUND_*` / `Context` /
# `InvalidOperation` layer DIVERGES + the class identity
# layer DIVERGES + the str(Decimal) value layer DIVERGES),
# and the full `numbers` module hasattr surface (the
# `isinstance(1, Integral)` / `isinstance(1.0, Real)`
# isinstance contract DIVERGES — mamba's ABCs do not
# register int / float).
#
# Surface in this fixture:
#   • math — full module hasattr surface (pi / e / tau /
#     inf / nan / sqrt / floor / ceil / trunc / log / log2
#     / log10 / exp / sin / cos / tan / asin / acos / atan
#     / atan2 / factorial / gcd / lcm / isfinite / isinf /
#     isnan / fabs / pow / hypot);
#   • math constants — pi / e / tau value contract;
#   • math functions — sqrt / floor / ceil / factorial /
#     gcd / isfinite / isinf / isnan / log value contract;
#   • cmath — partial module hasattr surface (pi / e /
#     sqrt / phase / polar / rect / exp / log / isnan /
#     isinf);
#   • cmath.sqrt(-1) / pi / phase — value contract;
#   • fractions — partial module hasattr surface (Fraction);
#   • fractions.Fraction — numerator / denominator
#     attribute contract;
#   • decimal — partial module hasattr surface (Decimal);
#   • numbers — full module hasattr surface (Number /
#     Complex / Real / Rational / Integral).
#
# Behavioral edges that DIVERGE on mamba
# (type(fractions.Fraction(1, 4)).__name__ returns "int"
# not "Fraction", Fraction(1, 4) + Fraction(1, 4) does not
# equal Fraction(1, 2) — the sum collapses to an integer
# placeholder, type(decimal.Decimal("0.1")).__name__
# returns "int" not "Decimal", str(decimal.Decimal("0.1"))
# returns a large integer string not "0.1", hasattr
# (decimal, "getcontext") / "setcontext" / "localcontext"
# / "ROUND_HALF_UP" / "ROUND_HALF_EVEN" / "ROUND_DOWN" /
# "ROUND_UP" / "ROUND_CEILING" / "ROUND_FLOOR" /
# "Context" / "InvalidOperation" all False, isinstance
# (1, numbers.Integral) returns False, isinstance(1.0,
# numbers.Real) returns False) are covered in the matching
# spec fixture `lang_fractions_decimal_numbers_silent`.
import math
import cmath
import fractions
import decimal
import numbers


_ledger: list[int] = []

# 1) math — full module hasattr surface
assert hasattr(math, "pi") == True; _ledger.append(1)
assert hasattr(math, "e") == True; _ledger.append(1)
assert hasattr(math, "tau") == True; _ledger.append(1)
assert hasattr(math, "inf") == True; _ledger.append(1)
assert hasattr(math, "nan") == True; _ledger.append(1)
assert hasattr(math, "sqrt") == True; _ledger.append(1)
assert hasattr(math, "floor") == True; _ledger.append(1)
assert hasattr(math, "ceil") == True; _ledger.append(1)
assert hasattr(math, "trunc") == True; _ledger.append(1)
assert hasattr(math, "log") == True; _ledger.append(1)
assert hasattr(math, "log2") == True; _ledger.append(1)
assert hasattr(math, "log10") == True; _ledger.append(1)
assert hasattr(math, "exp") == True; _ledger.append(1)
assert hasattr(math, "sin") == True; _ledger.append(1)
assert hasattr(math, "cos") == True; _ledger.append(1)
assert hasattr(math, "tan") == True; _ledger.append(1)
assert hasattr(math, "asin") == True; _ledger.append(1)
assert hasattr(math, "acos") == True; _ledger.append(1)
assert hasattr(math, "atan") == True; _ledger.append(1)
assert hasattr(math, "atan2") == True; _ledger.append(1)
assert hasattr(math, "factorial") == True; _ledger.append(1)
assert hasattr(math, "gcd") == True; _ledger.append(1)
assert hasattr(math, "lcm") == True; _ledger.append(1)
assert hasattr(math, "isfinite") == True; _ledger.append(1)
assert hasattr(math, "isinf") == True; _ledger.append(1)
assert hasattr(math, "isnan") == True; _ledger.append(1)
assert hasattr(math, "fabs") == True; _ledger.append(1)
assert hasattr(math, "pow") == True; _ledger.append(1)
assert hasattr(math, "hypot") == True; _ledger.append(1)

# 2) math constants — pi / e / tau value contract
assert math.pi == 3.141592653589793; _ledger.append(1)
assert math.e == 2.718281828459045; _ledger.append(1)
assert math.tau == 6.283185307179586; _ledger.append(1)

# 3) math functions — value contract
assert math.sqrt(16) == 4.0; _ledger.append(1)
assert math.floor(3.7) == 3; _ledger.append(1)
assert math.ceil(3.2) == 4; _ledger.append(1)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.gcd(12, 18) == 6; _ledger.append(1)
assert math.isfinite(1.0) == True; _ledger.append(1)
assert math.isnan(float("nan")) == True; _ledger.append(1)
assert math.isinf(float("inf")) == True; _ledger.append(1)
assert math.log(math.e) == 1.0; _ledger.append(1)

# 4) cmath — partial module hasattr surface
assert hasattr(cmath, "pi") == True; _ledger.append(1)
assert hasattr(cmath, "e") == True; _ledger.append(1)
assert hasattr(cmath, "sqrt") == True; _ledger.append(1)
assert hasattr(cmath, "phase") == True; _ledger.append(1)
assert hasattr(cmath, "polar") == True; _ledger.append(1)
assert hasattr(cmath, "rect") == True; _ledger.append(1)
assert hasattr(cmath, "exp") == True; _ledger.append(1)
assert hasattr(cmath, "log") == True; _ledger.append(1)
assert hasattr(cmath, "isnan") == True; _ledger.append(1)
assert hasattr(cmath, "isinf") == True; _ledger.append(1)

# 5) cmath — sqrt / pi / phase value contract
assert cmath.sqrt(-1) == 1j; _ledger.append(1)
assert cmath.pi == 3.141592653589793; _ledger.append(1)
assert cmath.phase(1j) == 1.5707963267948966; _ledger.append(1)

# 6) fractions — partial module hasattr surface
#    (class identity + arithmetic value DIVERGE — moved to
#    spec fixture)
assert hasattr(fractions, "Fraction") == True; _ledger.append(1)

# 7) fractions.Fraction — numerator / denominator attribute
_f = fractions.Fraction(1, 4)
assert _f.numerator == 1; _ledger.append(1)
assert _f.denominator == 4; _ledger.append(1)

# 8) decimal — partial module hasattr surface
#    (extended class / function / sentinel layer DIVERGES +
#    class identity layer DIVERGES + str(Decimal) value
#    layer DIVERGES — moved to spec fixture)
assert hasattr(decimal, "Decimal") == True; _ledger.append(1)

# 9) numbers — full module hasattr surface
#    (isinstance contract DIVERGES — moved to spec fixture)
assert hasattr(numbers, "Number") == True; _ledger.append(1)
assert hasattr(numbers, "Complex") == True; _ledger.append(1)
assert hasattr(numbers, "Real") == True; _ledger.append(1)
assert hasattr(numbers, "Rational") == True; _ledger.append(1)
assert hasattr(numbers, "Integral") == True; _ledger.append(1)

# NB: type(fractions.Fraction(1, 4)).__name__ returns "int"
# on mamba, Fraction(1, 4) + Fraction(1, 4) does not equal
# Fraction(1, 2) on mamba, type(decimal.Decimal("0.1"))
# .__name__ returns "int" on mamba, str(decimal.Decimal
# ("0.1")) returns a large integer string on mamba,
# hasattr(decimal, "getcontext") / "setcontext" /
# "localcontext" / "ROUND_HALF_UP" / "ROUND_HALF_EVEN" /
# "ROUND_DOWN" / "ROUND_UP" / "ROUND_CEILING" /
# "ROUND_FLOOR" / "Context" / "InvalidOperation" all False
# on mamba, isinstance(1, numbers.Integral) returns False
# on mamba, isinstance(1.0, numbers.Real) returns False on
# mamba — all DIVERGE on mamba — moved to the divergence-
# spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_math_cmath_fractions_decimal_numbers_value_ops {sum(_ledger)} asserts")
