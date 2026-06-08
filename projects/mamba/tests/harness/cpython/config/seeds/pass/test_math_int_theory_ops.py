# Operational AssertionPass seed for math module integer-theory
# functions (Py3.8+ surface).
# Surface:
#   • math.gcd(*ints) — variadic GCD over any number of ints,
#     with the two-arg form returning the absolute GCD and the
#     no-arg form returning 0;
#   • math.lcm(*ints) — variadic LCM, returning 0 if any operand
#     is 0;
#   • math.isqrt(n) — integer square root (floor of sqrt for n>=0);
#   • math.factorial(n) — n! for non-negative int;
#   • math.perm(n, k) — number of k-permutations of n distinct
#     items (n!/(n-k)!);
#   • math.comb(n, k) — binomial coefficient (n choose k).
#
# `math.perm(n)` (single-arg = n!) is deliberately NOT exercised
# here — mamba 0.3.60 returns 0 for the unary form (CPython
# returns n!); that gap moves to a focused spec/ seed.
import math
_ledger: list[int] = []

# gcd — two-arg form
assert math.gcd(12, 18) == 6; _ledger.append(1)
assert math.gcd(0, 5) == 5; _ledger.append(1)
assert math.gcd(5, 0) == 5; _ledger.append(1)
assert math.gcd(0, 0) == 0; _ledger.append(1)
assert math.gcd(17, 5) == 1; _ledger.append(1)  # coprime
assert math.gcd(100, 75) == 25; _ledger.append(1)

# gcd — variadic form (Py3.9+)
assert math.gcd(12, 18, 24) == 6; _ledger.append(1)
assert math.gcd(2, 4, 8, 16) == 2; _ledger.append(1)

# lcm — two-arg form (Py3.9+)
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.lcm(3, 5) == 15; _ledger.append(1)
assert math.lcm(0, 5) == 0; _ledger.append(1)  # 0 -> 0
assert math.lcm(7, 1) == 7; _ledger.append(1)

# lcm — variadic form
assert math.lcm(2, 3, 4) == 12; _ledger.append(1)
assert math.lcm(2, 4, 8) == 8; _ledger.append(1)

# isqrt — integer floor of sqrt
assert math.isqrt(0) == 0; _ledger.append(1)
assert math.isqrt(1) == 1; _ledger.append(1)
assert math.isqrt(4) == 2; _ledger.append(1)
assert math.isqrt(10) == 3; _ledger.append(1)  # floor(sqrt(10))
assert math.isqrt(16) == 4; _ledger.append(1)
assert math.isqrt(99) == 9; _ledger.append(1)
assert math.isqrt(100) == 10; _ledger.append(1)
assert math.isqrt(1_000_000) == 1000; _ledger.append(1)

# factorial — for small non-negative ints
assert math.factorial(0) == 1; _ledger.append(1)
assert math.factorial(1) == 1; _ledger.append(1)
assert math.factorial(2) == 2; _ledger.append(1)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.factorial(7) == 5040; _ledger.append(1)
assert math.factorial(10) == 3628800; _ledger.append(1)

# perm(n, k) — number of permutations
assert math.perm(5, 2) == 20; _ledger.append(1)  # 5*4
assert math.perm(5, 0) == 1; _ledger.append(1)
assert math.perm(5, 5) == 120; _ledger.append(1)  # 5!
assert math.perm(10, 3) == 720; _ledger.append(1)  # 10*9*8
assert math.perm(4, 1) == 4; _ledger.append(1)

# comb(n, k) — binomial coefficient
assert math.comb(5, 0) == 1; _ledger.append(1)
assert math.comb(5, 5) == 1; _ledger.append(1)
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.comb(5, 3) == 10; _ledger.append(1)  # symmetric with comb(5,2)
assert math.comb(10, 3) == 120; _ledger.append(1)
assert math.comb(20, 10) == 184756; _ledger.append(1)
assert math.comb(1, 0) == 1; _ledger.append(1)
assert math.comb(0, 0) == 1; _ledger.append(1)

# Combinatorial identity: comb(n, k) == comb(n, n-k)
assert math.comb(10, 3) == math.comb(10, 7); _ledger.append(1)
assert math.comb(8, 2) == math.comb(8, 6); _ledger.append(1)

# perm(n, k) == comb(n, k) * factorial(k)
assert math.perm(5, 2) == math.comb(5, 2) * math.factorial(2); _ledger.append(1)
assert math.perm(7, 3) == math.comb(7, 3) * math.factorial(3); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_math_int_theory_ops {sum(_ledger)} asserts")
