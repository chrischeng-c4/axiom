# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/float_methods: behavior asserts (CPython 3.12 oracle)."""

import math

# Core arithmetic and mixed int/float promotion.
assert 1.5 + 2.5 == 4.0
assert 5 / 2 == 2.5          # true division is always float in Py3
assert 7.5 // 2.0 == 3.0
assert 7.5 % 2.0 == 1.5
assert 2.0 ** 10 == 1024.0
assert type(1 + 2.0) is float

# Conversions both directions.
assert float("3.14") == 3.14
assert float(True) == 1.0
assert int(3.9) == 3          # truncates toward zero
assert int(-3.9) == -3
assert str(2.5) == "2.5"

# Classic floating-point imprecision is observable.
assert 0.1 + 0.2 != 0.3
assert abs((0.1 + 0.2) - 0.3) < 1e-10

# Signed zero compares equal but keeps its sign.
assert 0.0 == -0.0
assert math.copysign(1.0, -0.0) == -1.0

# is_integer distinguishes whole-valued floats.
assert (4.0).is_integer()
assert not (4.5).is_integer()

# Special values behave per IEEE 754.
assert math.isinf(float("inf"))
assert math.isnan(float("nan"))
assert float("nan") != float("nan")   # NaN is never equal to itself

print("behavior OK")
