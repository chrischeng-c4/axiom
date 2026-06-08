# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# float.as_integer_ratio — pairs that fit mamba's i48 int range.
# Inputs whose CPython numerator/denominator exceeds 2**47 (e.g. 0.1, 3.14)
# raise OverflowError on mamba; that's tracked separately as the i48-bound
# limitation and intentionally not exercised here.

print((1.5).as_integer_ratio())
print((2.0).as_integer_ratio())
print((3.0).as_integer_ratio())
print((100.0).as_integer_ratio())
print((1024.0).as_integer_ratio())

# Negatives.
print((-1.5).as_integer_ratio())
print((-2.0).as_integer_ratio())
print((-100.0).as_integer_ratio())

# Powers of two — denominator is 1.
print((4.0).as_integer_ratio())
print((8.0).as_integer_ratio())
print((1.0).as_integer_ratio())
print((0.5).as_integer_ratio())
print((0.25).as_integer_ratio())
print((0.125).as_integer_ratio())

# Signed zero collapses to (0, 1).
print((0.0).as_integer_ratio())
print((-0.0).as_integer_ratio())

# Small dyadic rationals.
print((1.75).as_integer_ratio())
print((-3.5).as_integer_ratio())
print((0.75).as_integer_ratio())

# NaN / Inf raise.
import math
try:
    math.nan.as_integer_ratio()
    print("expected ValueError")
except ValueError:
    print("ValueError on nan")

try:
    math.inf.as_integer_ratio()
    print("expected OverflowError")
except OverflowError:
    print("OverflowError on inf")

try:
    (-math.inf).as_integer_ratio()
    print("expected OverflowError")
except OverflowError:
    print("OverflowError on -inf")

# Round-trip: f == num / den (within float eps).
for f in [1.5, 2.0, -3.5, 0.25, 0.75]:
    pair = f.as_integer_ratio()
    num = pair[0]
    den = pair[1]
    print(num / den == f)
