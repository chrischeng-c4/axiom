# Operational AssertionPass seed for math module surfaces beyond
# test_math_extras_ops / test_math_float_ops.
# Surface: combinatoric helpers (factorial, comb, perm, prod); number
# theory (gcd, lcm); floating-point predicates (isnan, isinf,
# isfinite); rounding family (trunc, ceil, floor); sign and magnitude
# (fabs, copysign, fmod); geometric helpers (hypot, dist).
import math
_ledger: list[int] = []

# Combinatoric helpers
assert math.factorial(0) == 1; _ledger.append(1)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.factorial(6) == 720; _ledger.append(1)

# comb(n, k) is the binomial coefficient "n choose k"
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.comb(10, 0) == 1; _ledger.append(1)
assert math.comb(10, 10) == 1; _ledger.append(1)

# perm(n, k) is the number of k-permutations of n distinct items
assert math.perm(5, 2) == 20; _ledger.append(1)
assert math.perm(5, 5) == 120; _ledger.append(1)

# prod over a list of ints
assert math.prod([1, 2, 3, 4]) == 24; _ledger.append(1)
# prod of an empty sequence is 1 (multiplicative identity)
assert math.prod([]) == 1; _ledger.append(1)

# gcd / lcm
assert math.gcd(12, 18) == 6; _ledger.append(1)
assert math.gcd(7, 13) == 1; _ledger.append(1)
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.lcm(3, 7) == 21; _ledger.append(1)

# Floating-point predicates
assert math.isnan(math.nan) == True; _ledger.append(1)
assert math.isnan(1.0) == False; _ledger.append(1)
assert math.isinf(math.inf) == True; _ledger.append(1)
assert math.isinf(1.0) == False; _ledger.append(1)
assert math.isfinite(1.0) == True; _ledger.append(1)
assert math.isfinite(math.inf) == False; _ledger.append(1)
assert math.isfinite(math.nan) == False; _ledger.append(1)

# Rounding family
assert math.trunc(3.7) == 3; _ledger.append(1)
# trunc toward zero — negative input truncates upward (toward 0)
assert math.trunc(-3.7) == -3; _ledger.append(1)
assert math.ceil(3.2) == 4; _ledger.append(1)
assert math.ceil(3.0) == 3; _ledger.append(1)
assert math.floor(3.8) == 3; _ledger.append(1)
assert math.floor(3.0) == 3; _ledger.append(1)

# Sign and magnitude
assert math.fabs(-5) == 5.0; _ledger.append(1)
assert math.fabs(5) == 5.0; _ledger.append(1)
# copysign returns |x| with the sign of y
assert math.copysign(3, -1) == -3.0; _ledger.append(1)
assert math.copysign(-3, 1) == 3.0; _ledger.append(1)
# fmod returns a result with the sign of the dividend (unlike Python's %)
assert math.fmod(10, 3) == 1.0; _ledger.append(1)

# Geometric helpers
assert math.hypot(3, 4) == 5.0; _ledger.append(1)
assert math.dist([0, 0], [3, 4]) == 5.0; _ledger.append(1)
# hypot of one number is its absolute value
assert math.hypot(5) == 5.0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_math_combinatoric_ops {sum(_ledger)} asserts")
