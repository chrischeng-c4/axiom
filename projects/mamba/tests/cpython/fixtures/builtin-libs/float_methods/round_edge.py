# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""round(float, ndigits) edge cases: int result, banker's, inf/nan, huge n."""

import math

INF = float("inf")
NAN = float("nan")

# round() with no ndigits (or None) returns an int, rounding half-to-even.
for x in (round(1.23), round(1.23, None), round(1.23, ndigits=None)):
    assert x == 1 and isinstance(x, int)
for x in (round(1.78), round(1.78, None), round(1.78, ndigits=None)):
    assert x == 2 and isinstance(x, int)

# round() uses banker's (round-half-to-even) at .5 ties.
assert round(0.5) == 0
assert round(1.5) == 2
assert round(2.5) == 2
assert round(3.5) == 4

# Half-to-even also applies at the tens place via negative ndigits.
assert round(25.0, -1) == 20.0
assert round(35.0, -1) == 40.0
assert round(45.0, -1) == 40.0
assert round(55.0, -1) == 60.0

# Values that cannot change at the requested precision are returned exactly.
assert round(562949953421312.5, 1) == 562949953421312.5

# Enormous ndigits is a no-op; result stays a float.
for n in (324, 400, 2 ** 31, 2 ** 100):
    assert round(123.456, n) == 123.456
    assert round(-123.456, n) == -123.456
    assert round(1e300, n) == 1e300

# inf / nan: with ndigits they pass through; without, they raise.
assert round(INF, 0) == INF
assert round(-INF, 0) == -INF
assert math.isnan(round(NAN, 0))

try:
    round(INF)
    raise AssertionError("expected OverflowError")
except OverflowError:
    pass
try:
    round(NAN)
    raise AssertionError("expected ValueError")
except ValueError:
    pass

# Rounding that would overflow the double range raises OverflowError.
try:
    round(1.6e308, -308)
    raise AssertionError("expected OverflowError")
except OverflowError:
    pass

# A non-integer ndigits is a TypeError.
try:
    round(1.0, 0.5)
    raise AssertionError("expected TypeError")
except TypeError:
    pass

print("round_edge OK")
