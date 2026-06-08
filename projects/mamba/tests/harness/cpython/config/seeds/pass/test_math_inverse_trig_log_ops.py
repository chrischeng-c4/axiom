# Operational AssertionPass seed for `math` inverse-trigonometric
# functions and arbitrary-base logarithm not asserted by existing
# math seeds. The seeds `test_math_ops`, `test_math_constants_trig_ops`,
# `test_math_extras_ops`, etc. cover sin/cos/tan and the
# hyperbolic-inverse trio (asinh/acosh/atanh) but not the regular
# inverse-trig trio (asin/acos/atan). Surface: asin(±1)=±pi/2,
# asin(0)=0; acos(1)=0, acos(0)=pi/2, acos(-1)=pi; atan(0)=0,
# atan(±inf)=±pi/2. Companion: `math.log(x, base)` over multiple
# (x, base) pairs that give exact integer results.
import math
_ledger: list[int] = []

# asin — inverse sine on [-1, 1] → [-pi/2, pi/2]
assert math.asin(0) == 0.0; _ledger.append(1)
assert math.asin(1) == math.pi / 2; _ledger.append(1)
assert math.asin(-1) == -math.pi / 2; _ledger.append(1)
# asin is odd: asin(-x) = -asin(x)
assert math.asin(-0.5) == -math.asin(0.5); _ledger.append(1)

# acos — inverse cosine on [-1, 1] → [0, pi]
assert math.acos(1) == 0.0; _ledger.append(1)
assert math.acos(0) == math.pi / 2; _ledger.append(1)
assert math.acos(-1) == math.pi; _ledger.append(1)
# acos(x) + asin(x) == pi/2
assert abs(math.acos(0.3) + math.asin(0.3) - math.pi / 2) < 1e-12; _ledger.append(1)

# atan — inverse tangent on R → (-pi/2, pi/2)
assert math.atan(0) == 0.0; _ledger.append(1)
assert math.atan(math.inf) == math.pi / 2; _ledger.append(1)
assert math.atan(-math.inf) == -math.pi / 2; _ledger.append(1)
# atan(1) == pi/4
assert math.atan(1) == math.pi / 4; _ledger.append(1)

# Round-trip: sin(asin(x)) ≈ x for x in [-1, 1]
assert abs(math.sin(math.asin(0.5)) - 0.5) < 1e-12; _ledger.append(1)
assert abs(math.cos(math.acos(0.5)) - 0.5) < 1e-12; _ledger.append(1)
assert abs(math.tan(math.atan(1.0)) - 1.0) < 1e-12; _ledger.append(1)

# log(x, base) — integer-result corners
assert math.log(8, 2) == 3.0; _ledger.append(1)
assert math.log(81, 3) == 4.0; _ledger.append(1)
assert math.log(16, 4) == 2.0; _ledger.append(1)
# log(125, 5) is mathematically 3.0 but floating-point yields ~3+epsilon
assert abs(math.log(125, 5) - 3.0) < 1e-12; _ledger.append(1)
# log(1, anything) == 0
assert math.log(1, 2) == 0.0; _ledger.append(1)
assert math.log(1, 10) == 0.0; _ledger.append(1)
# log(b, b) == 1 for any positive base != 1
assert math.log(2, 2) == 1.0; _ledger.append(1)
assert math.log(7, 7) == 1.0; _ledger.append(1)

# Identity: log(x, base) == log(x) / log(base)
assert abs(math.log(50, 3) - math.log(50) / math.log(3)) < 1e-12; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_math_inverse_trig_log_ops {sum(_ledger)} asserts")
