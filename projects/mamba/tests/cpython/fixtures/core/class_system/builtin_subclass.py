# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Subclassing builtin types and reflected-operator subclass priority."""


# str subclass keeps str behavior while overriding __eq__/__hash__.
class CIStr(str):
    def __new__(cls, value):
        return super().__new__(cls, value)

    @property
    def canonical(self):
        return self.lower()

    def __eq__(self, other):
        if not isinstance(other, CIStr):
            other = CIStr(other)
        return self.canonical == other.canonical

    def __hash__(self):
        return hash(self.canonical)


assert CIStr("ABC") == "abc"
assert "aBc" == CIStr("ABC")
assert str(CIStr("ABC")) == "ABC"        # underlying str value preserved
assert isinstance(CIStr("x"), str)

# Case-insensitive instances collapse as dict keys.
d = {CIStr("One"): 1, CIStr("Two"): 2}
assert d[CIStr("ONE")] == 1
assert d[CIStr("two")] == 2
assert CIStr("oNe") in d


# int subclass overriding both forward and reflected floordiv.
class B(int):
    def __floordiv__(self, other):
        return "B.fl"

    def __rfloordiv__(self, other):
        return "B.rfl"


assert B(1) // 1 == "B.fl"
assert 1 // B(1) == "B.rfl"


# When the right operand's type is a subclass of the left operand's type,
# the subclass's reflected method is tried first.
class P:
    def __floordiv__(self, other):
        return "P.fl"

    def __rfloordiv__(self, other):
        return "P.rfl"


class Q(P):
    def __floordiv__(self, other):
        return "Q.fl"

    def __rfloordiv__(self, other):
        return "Q.rfl"


assert Q() // P() == "Q.fl"
assert P() // Q() == "Q.rfl"     # subclass reflected op wins


# complex subclass compares equal to the equivalent complex value.
class Z(complex):
    pass


assert Z(1) == 1 + 0j
assert 1 + 0j == Z(1)

# Explicit reflected dunders on builtins are directly callable.
assert complex.__radd__(3j, 4.0) == complex(4.0, 3.0)
assert float.__rsub__(3.0, 1) == -2.0

print("builtin_subclass OK")
