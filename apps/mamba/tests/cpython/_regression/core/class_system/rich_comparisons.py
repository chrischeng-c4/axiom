# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Rich comparison dunders and reflected fallbacks (CPython 3.12 oracle)."""


class Num:
    def __init__(self, value):
        self.value = value

    def _other(self, o):
        return o.value if isinstance(o, Num) else o

    def __eq__(self, o):
        return self.value == self._other(o)

    def __ne__(self, o):
        return self.value != self._other(o)

    def __lt__(self, o):
        return self.value < self._other(o)

    def __le__(self, o):
        return self.value <= self._other(o)

    def __gt__(self, o):
        return self.value > self._other(o)

    def __ge__(self, o):
        return self.value >= self._other(o)


a, b = Num(1), Num(2)

# Comparisons between two user instances.
assert a < b and a <= b and b > a and b >= a
assert a == Num(1) and a != b

# Comparison against a plain int on the left: Num.__lt__ handles it.
assert a < 5
assert Num(3) >= 3
assert Num(3) == 3

# Reflected fallback: int does not know Num, so `5 < Num(7)` becomes
# Num(7).__gt__(5). Confirms Python tries the reflected operator.
assert 5 < Num(7)
assert 5 > Num(2)
assert 4 == Num(4)

# __ne__ is honored independently (not auto-derived here).
assert (a != Num(1)) is False
assert (a != b) is True

print("rich_comparisons OK")
