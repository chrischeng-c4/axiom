# Operational AssertionPass seed for math surfaces beyond test_math_*_ops
# (float / extras / combinatoric trio). Surface: math constants e / pi
# / tau / inf / nan; classifiers isinf / isnan / isfinite; gcd (two,
# zero-operand, coprime, three-arg) and lcm; factorial (0, small,
# medium); comb (n, k) and perm (n, k); the log family — log (natural
# and explicit-base), log10, log2, exp; trig — sin/cos/tan at 0 and
# at pi/2 / pi; atan2 (returns angle in [-pi, pi]); hypot; sqrt; pow
# (math, returns float); floor / ceil / trunc on positive AND negative
# floats (trunc towards zero); degrees and radians (with the obvious
# round-trip relationship); copysign and fabs.
import math
_ledger: list[int] = []

# Constants (compared with a tolerance — these are floats, not literal
# 3.141592...)
assert abs(math.e - 2.718281828459045) < 1e-10; _ledger.append(1)
assert abs(math.pi - 3.141592653589793) < 1e-10; _ledger.append(1)
assert abs(math.tau - 2 * math.pi) < 1e-10; _ledger.append(1)
assert math.inf > 1e308; _ledger.append(1)
# A NaN value is never equal to itself — this is the defining
# property of NaN under IEEE 754
assert math.nan != math.nan; _ledger.append(1)

# Classifiers
assert math.isinf(math.inf); _ledger.append(1)
assert math.isnan(math.nan); _ledger.append(1)
assert math.isfinite(1.0); _ledger.append(1)
assert not math.isfinite(math.inf); _ledger.append(1)
assert not math.isfinite(math.nan); _ledger.append(1)

# gcd — two-operand; with 0 returns the other argument; coprime returns
# 1; three-or-more arguments
assert math.gcd(12, 8) == 4; _ledger.append(1)
assert math.gcd(0, 5) == 5; _ledger.append(1)
assert math.gcd(7, 11) == 1; _ledger.append(1)
assert math.gcd(12, 18, 24) == 6; _ledger.append(1)

# lcm
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.lcm(3, 5) == 15; _ledger.append(1)

# factorial — including 0! = 1
assert math.factorial(0) == 1; _ledger.append(1)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.factorial(10) == 3628800; _ledger.append(1)

# comb — binomial coefficient C(n, k)
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.comb(10, 3) == 120; _ledger.append(1)
assert math.comb(0, 0) == 1; _ledger.append(1)

# perm — permutation count P(n, k) (two-arg form)
assert math.perm(5, 2) == 20; _ledger.append(1)
assert math.perm(10, 3) == 720; _ledger.append(1)

# Log family
assert abs(math.log(math.e) - 1.0) < 1e-10; _ledger.append(1)
assert math.log(1) == 0.0; _ledger.append(1)
assert math.log10(100) == 2.0; _ledger.append(1)
assert math.log2(8) == 3.0; _ledger.append(1)
# Explicit-base log
assert abs(math.log(100, 10) - 2.0) < 1e-10; _ledger.append(1)
assert math.exp(0) == 1.0; _ledger.append(1)
assert abs(math.exp(1) - math.e) < 1e-10; _ledger.append(1)

# Trig — sin/cos/tan at 0 are exact
assert math.sin(0) == 0.0; _ledger.append(1)
assert math.cos(0) == 1.0; _ledger.append(1)
assert math.tan(0) == 0.0; _ledger.append(1)
# sin(pi/2) ≈ 1.0
assert abs(math.sin(math.pi / 2) - 1.0) < 1e-10; _ledger.append(1)
# cos(pi) ≈ -1.0
assert abs(math.cos(math.pi) - (-1.0)) < 1e-10; _ledger.append(1)

# atan2 / hypot
assert abs(math.atan2(1, 1) - math.pi / 4) < 1e-10; _ledger.append(1)
assert math.hypot(3, 4) == 5.0; _ledger.append(1)

# sqrt / pow — pow is the math (float) version, not the builtin
assert math.sqrt(16) == 4.0; _ledger.append(1)
assert abs(math.sqrt(2) - 1.4142135623730951) < 1e-10; _ledger.append(1)
assert math.pow(2, 10) == 1024.0; _ledger.append(1)

# floor / ceil / trunc — trunc rounds towards zero (not towards -inf)
assert math.floor(3.7) == 3; _ledger.append(1)
assert math.ceil(3.2) == 4; _ledger.append(1)
assert math.trunc(3.7) == 3; _ledger.append(1)
assert math.trunc(-3.7) == -3; _ledger.append(1)
assert math.floor(-3.2) == -4; _ledger.append(1)
assert math.ceil(-3.7) == -3; _ledger.append(1)

# degrees / radians — round-trip relationship
assert math.degrees(math.pi) == 180.0; _ledger.append(1)
assert abs(math.radians(180) - math.pi) < 1e-10; _ledger.append(1)
assert math.degrees(0) == 0.0; _ledger.append(1)
assert math.radians(0) == 0.0; _ledger.append(1)

# copysign / fabs
assert math.copysign(3, -1) == -3.0; _ledger.append(1)
assert math.copysign(-3, 1) == 3.0; _ledger.append(1)
assert math.fabs(-5.5) == 5.5; _ledger.append(1)
assert math.fabs(5.5) == 5.5; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_math_constants_trig_ops {sum(_ledger)} asserts")
