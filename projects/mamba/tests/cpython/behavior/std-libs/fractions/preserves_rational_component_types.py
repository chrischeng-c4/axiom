# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "preserves_rational_component_types"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: built from objects exposing .numerator/.denominator, Fraction divides through by the gcd but keeps the original component type rather than coercing to plain int (int subclass MyInt)"""
from fractions import Fraction


class MyInt(int):
    """An int that returns its own type from arithmetic and components."""

    def __mul__(self, other):
        return type(self)(int(self) * int(other))

    def __floordiv__(self, other):
        return type(self)(int(self) // int(other))

    def __mod__(self, other):
        return type(self)(int(self) % int(other))

    @property
    def numerator(self):
        return type(self)(int(self))

    @property
    def denominator(self):
        return type(self)(1)


# 3/6 reduces to 1/2 while retaining MyInt components.
f = Fraction(MyInt(1 * 3), MyInt(2 * 3))
assert f.numerator == 1, f"numerator value = {f.numerator!r}"
assert f.denominator == 2, f"denominator value = {f.denominator!r}"
assert type(f.numerator) is MyInt, f"numerator type = {type(f.numerator)!r}"
assert type(f.denominator) is MyInt, f"denominator type = {type(f.denominator)!r}"

print("preserves_rational_component_types OK")
