# Operational AssertionPass seed for the value contract of the
# two pure-numeric bootstrap stdlib modules: `math` (the
# documented real-valued constants + arithmetic / power /
# logarithm / trigonometric / floor / ceil / trunc / abs /
# gcd / lcm / factorial / combinatorics / NaN-and-inf
# predicates / fabs / copysign / fmod / hypot / dist / exp
# helpers) and `cmath` (the documented complex-valued
# equivalents — pi / e / inf + sqrt of negatives + polar +
# rect + phase + exp + NaN/inf predicates).
#
# The matching subset between mamba and CPython is the entire
# documented numeric-helper surface: every value below matches
# byte-for-byte on both runtimes — math.pi to 17 significant
# digits; math.sqrt(16) == 4.0; math.gcd(12, 8) == 4;
# math.factorial(5) == 120; math.comb(5, 2) == 10;
# math.isclose(1, 1.0000001) == False (default rel_tol 1e-9);
# math.dist([0,0], [3,4]) == 5.0; cmath.sqrt(-1) == 1j;
# cmath.polar(1+1j) == (sqrt(2), pi/4); cmath.phase(1j) ==
# pi/2.
#
# Surface in this fixture:
#   • math constants — pi / e / tau / inf;
#   • math NaN — nan != nan (IEEE-754 reflexive-inequality);
#   • math.sqrt / pow / log / log10 / log2 + log identity;
#   • math.sin / cos at 0;
#   • math.floor / ceil / trunc / abs;
#   • math.gcd / lcm / factorial / comb / perm;
#   • math.isnan / isinf / isfinite / isclose predicates;
#   • math.copysign / fabs / fmod / hypot / dist / exp /
#     expm1;
#   • cmath constants pi / e / inf + nan != nan;
#   • cmath.sqrt of negative + polar / rect / phase / exp;
#   • cmath.isnan / isinf predicates.
#
# Behavioral edges that DIVERGE on mamba (decimal.Decimal
# returns an integer handle, decimal.getcontext().prec None,
# decimal.ROUND_HALF_EVEN / ROUND_UP / ROUND_DOWN None,
# decimal.Decimal.quantize AttributeError, fractions.Fraction
# returns an integer handle, fractions.Fraction class
# identity, numbers.Number / Complex / Real / Rational /
# Integral class identity, isinstance(1, numbers.Integral) /
# isinstance(1.5, numbers.Real) returning False instead of
# True — the documented numeric tower) are covered in the
# matching spec fixture `lang_decimal_fractions_numbers_silent`.
import math
import cmath


_ledger: list[int] = []

# 1) math — pi to 17 significant digits
assert math.pi == 3.141592653589793; _ledger.append(1)

# 2) math — Euler's number to 17 significant digits
assert math.e == 2.718281828459045; _ledger.append(1)

# 3) math — tau == 2 * pi
assert math.tau == 6.283185307179586; _ledger.append(1)

# 4) math — positive infinity literal
assert math.inf > 1e300; _ledger.append(1)

# 5) math — IEEE-754 NaN reflexive inequality
assert (math.nan != math.nan) == True; _ledger.append(1)

# 6) math — sqrt / pow
assert math.sqrt(16) == 4.0; _ledger.append(1)
assert math.pow(2, 10) == 1024.0; _ledger.append(1)

# 7) math — logarithms
assert math.log(math.e) == 1.0; _ledger.append(1)
assert math.log10(1000) == 3.0; _ledger.append(1)
assert math.log2(8) == 3.0; _ledger.append(1)

# 8) math — trigonometry at 0
assert math.sin(0) == 0.0; _ledger.append(1)
assert math.cos(0) == 1.0; _ledger.append(1)

# 9) math — floor / ceil / trunc
assert math.floor(3.7) == 3; _ledger.append(1)
assert math.ceil(3.2) == 4; _ledger.append(1)
assert math.trunc(3.9) == 3; _ledger.append(1)

# 10) math — abs
assert abs(-2.5) == 2.5; _ledger.append(1)

# 11) math — gcd / lcm
assert math.gcd(12, 8) == 4; _ledger.append(1)
assert math.lcm(4, 6) == 12; _ledger.append(1)

# 12) math — factorial / combinatorics
assert math.factorial(5) == 120; _ledger.append(1)
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.perm(5, 2) == 20; _ledger.append(1)

# 13) math — NaN / inf / finite predicates
assert math.isnan(math.nan) == True; _ledger.append(1)
assert math.isinf(math.inf) == True; _ledger.append(1)
assert math.isfinite(1) == True; _ledger.append(1)
assert math.isclose(1, 1.0000001) == False; _ledger.append(1)

# 14) math — signed / absolute / floating-modulo helpers
assert math.copysign(3, -1) == -3.0; _ledger.append(1)
assert math.fabs(-2.5) == 2.5; _ledger.append(1)
assert math.fmod(7, 3) == 1.0; _ledger.append(1)

# 15) math — Euclidean helpers
assert math.hypot(3, 4) == 5.0; _ledger.append(1)
assert math.dist([0, 0], [3, 4]) == 5.0; _ledger.append(1)

# 16) math — exp / expm1
assert math.exp(1) == 2.718281828459045; _ledger.append(1)
assert math.expm1(1) == 1.7182818284590453; _ledger.append(1)

# 17) cmath — pi / e / inf
assert cmath.pi == 3.141592653589793; _ledger.append(1)
assert cmath.e == 2.718281828459045; _ledger.append(1)
assert cmath.inf > 1e300; _ledger.append(1)
assert (cmath.nan != cmath.nan) == True; _ledger.append(1)

# 18) cmath — sqrt of negative real
assert cmath.sqrt(-1) == 1j; _ledger.append(1)

# 19) cmath — polar / rect / phase
assert cmath.polar(1 + 1j) == (1.4142135623730951, 0.7853981633974483); _ledger.append(1)
assert cmath.rect(1, 0) == (1 + 0j); _ledger.append(1)
assert cmath.phase(1j) == 1.5707963267948966; _ledger.append(1)

# 20) cmath — NaN / inf predicates over complex
assert cmath.isnan(complex("nan")) == True; _ledger.append(1)
assert cmath.isinf(complex("inf")) == True; _ledger.append(1)

# NB: decimal.Decimal integer-handle behaviour,
# decimal.getcontext().prec None, decimal ROUND_HALF_EVEN /
# ROUND_UP / ROUND_DOWN None, decimal.Decimal.quantize
# AttributeError, fractions.Fraction integer-handle behaviour,
# fractions.Fraction class identity, numbers.Number / Complex
# / Real / Rational / Integral class identity, isinstance(1,
# numbers.Integral) / isinstance(1.5, numbers.Real) — the
# documented numeric tower — all DIVERGE on mamba and are
# covered in the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_math_cmath_numeric_value_ops {sum(_ledger)} asserts")
