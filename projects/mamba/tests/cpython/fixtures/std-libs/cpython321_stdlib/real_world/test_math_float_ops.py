# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_math_float_ops"
# subject = "cpython321.test_math_float_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_math_float_ops.py"
# status = "filled"
# ///
"""cpython321.test_math_float_ops: execute CPython 3.12 seed test_math_float_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for math float-introspection +
# integer-arithmetic surface not covered by test_math_extras_ops.py.
# Surface: lcm, isfinite, isnan, isinf, copysign, trunc, modf.
import math
_ledger: list[int] = []
# lcm: smallest common multiple of two positives
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.lcm(7, 5) == 35; _ledger.append(1)
# isfinite returns True only for finite floats
assert math.isfinite(1.0); _ledger.append(1)
assert math.isfinite(0.0); _ledger.append(1)
assert not math.isfinite(float("inf")); _ledger.append(1)
# isinf is the complement on the infinity axis
assert math.isinf(float("inf")); _ledger.append(1)
assert not math.isinf(1.0); _ledger.append(1)
# isnan recognises NaN; NaN is not finite, not equal to itself
assert math.isnan(float("nan")); _ledger.append(1)
assert not math.isnan(1.0); _ledger.append(1)
# copysign carries the sign of the second arg onto the magnitude of the first
assert math.copysign(3.0, -1.0) == -3.0; _ledger.append(1)
assert math.copysign(-3.0, 1.0) == 3.0; _ledger.append(1)
# trunc drops the fractional part toward zero (returns int)
assert math.trunc(3.7) == 3; _ledger.append(1)
assert math.trunc(-3.7) == -3; _ledger.append(1)
# modf splits a float into (fractional_part, integer_part)
frac, intp = math.modf(3.5)
assert frac == 0.5; _ledger.append(1)
assert intp == 3.0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_math_float_ops {sum(_ledger)} asserts")
