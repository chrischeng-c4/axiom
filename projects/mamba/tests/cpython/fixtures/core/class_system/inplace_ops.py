# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""In-place operator dunders and NotImplemented fallback (CPython 3.12)."""


# __imul__ may return any object; that object rebinds the name.
class Capture:
    def __imul__(self, other):
        return (self, other)


x = Capture()
orig = x
x *= 2
assert x == (orig, 2)

y = Capture()
o2 = y
y *= "foo"
assert y == (o2, "foo")


# __ipow__ returning None means the augmented assignment yields None.
class PowNone:
    def __ipow__(self, other):
        return None


z = PowNone()
z **= 3
assert z is None


# __ipow__ returning NotImplemented falls back to normal __pow__/__rpow__.
class Base:
    def __ipow__(self, other):
        return NotImplemented


class HasRpow(Base):
    def __rpow__(self, other):
        return "rpow"


class HasPow(Base):
    def __pow__(self, other):
        return "pow"


a = Base()
a **= HasRpow()          # a.__ipow__ -> NotImplemented, then HasRpow.__rpow__
assert a == "rpow"

c = HasPow()
c **= HasRpow()          # c.__ipow__ -> NotImplemented, then c.__pow__
assert c == "pow"


# Augmented assignment without an in-place dunder uses the binary dunder.
class Adder:
    def __init__(self, v):
        self.v = v

    def __add__(self, other):
        return Adder(self.v + other)


acc = Adder(1)
acc += 4
assert acc.v == 5

print("inplace_ops OK")
