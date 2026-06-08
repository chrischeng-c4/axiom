# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/builtins: behavior asserts (CPython 3.12 oracle)."""

# Verify a few core behaviors match CPython expectations.
assert 1 + 1 == 2
assert "a" + "b" == "ab"
assert [1, 2] + [3] == [1, 2, 3]
assert sorted([3, 1, 2]) == [1, 2, 3]
assert len("abc") == 3

# sum() is order-accurate on these float inputs (matches CPython exactly).
assert sum([0.1] * 10) == 1.0
assert sum([1.0, 1e101, 1.0, -1e101]) == 2.0
assert sum([1, 2, 3], 10) == 16   # start value

# divmod() satisfies a == b * q + r with the sign of the divisor on r.
assert divmod(12, 7) == (1, 5)
assert divmod(-12, 7) == (-2, 2)
assert divmod(12, -7) == (-2, -2)
assert divmod(-12, -7) == (1, -5)
# Float divmod returns float quotient and remainder.
fq, fr = divmod(3.25, 1.0)
assert abs(fq - 3.0) < 1e-9 and abs(fr - 0.25) < 1e-9

# round() with a custom __round__ delegates to it.
class Rounder:
    def __round__(self):
        return 23


assert round(Rounder()) == 23

# Built-in singletons construct to themselves.
assert type(None)() is None
assert type(...)() is Ellipsis

# sorted() accepts the same content from any iterable type.
src = "abracadabra"
assert sorted(src) == sorted(tuple(src)) == sorted(list(src))

print("behavior OK")
