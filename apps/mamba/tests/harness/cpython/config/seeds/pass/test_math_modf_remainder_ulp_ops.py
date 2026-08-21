# Operational AssertionPass seed for `math` functions not covered
# by `test_math_ops`, `test_math_constants_trig_ops`,
# `test_math_extras_ops`, `test_math_float_ops`, or
# `test_math_int_theory_ops`. Surface: `math.modf(x)` returns the
# `(fractional, integer-part-as-float)` decomposition;
# `math.remainder(x, y)` returns the IEEE-754 remainder;
# `math.nextafter(x, y)` returns the next representable float
# toward `y`; `math.ulp(x)` returns the unit in last place at `x`
# (strictly positive). Companion surface: `math.fmod` over a
# negative dividend keeps the sign of the dividend (-10 % 3 → -1.0
# in C-style fmod, vs Python `%` which keeps divisor sign).
import math
_ledger: list[int] = []

# modf — fractional / integer-part split
frac, whole = math.modf(3.75)
assert frac == 0.75; _ledger.append(1)
assert whole == 3.0; _ledger.append(1)

frac2, whole2 = math.modf(-2.25)
assert frac2 == -0.25; _ledger.append(1)
assert whole2 == -2.0; _ledger.append(1)

# remainder — IEEE-754 remainder (sign follows divisor, not dividend)
assert math.remainder(7, 3) == 1.0; _ledger.append(1)
assert math.remainder(8, 3) == -1.0; _ledger.append(1)

# nextafter — moves toward the second arg by one ULP
assert math.nextafter(1.0, 2.0) > 1.0; _ledger.append(1)
assert math.nextafter(1.0, 0.0) < 1.0; _ledger.append(1)
assert math.nextafter(1.0, 1.0) == 1.0; _ledger.append(1)

# ulp — unit in last place, strictly positive on finite normal values
assert math.ulp(1.0) > 0; _ledger.append(1)
assert math.ulp(2.0) > 0; _ledger.append(1)
assert math.ulp(1.0) < 1.0; _ledger.append(1)

# fmod — C-style remainder (sign of dividend)
assert math.fmod(10, 3) == 1.0; _ledger.append(1)
assert math.fmod(-10, 3) == -1.0; _ledger.append(1)
assert math.fmod(10, -3) == 1.0; _ledger.append(1)

# copysign — magnitude of arg1, sign of arg2 (extra corners)
assert math.copysign(0.0, -1.0) == 0.0; _ledger.append(1)
assert math.copysign(5.0, -0.0) == -5.0; _ledger.append(1)
assert math.copysign(5.0, 0.0) == 5.0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_math_modf_remainder_ulp_ops {sum(_ledger)} asserts")
