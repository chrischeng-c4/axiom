# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: `(5).__add__(3)` and friends — arithmetic dunders were
# missing from the primitive method dispatcher, so calling them
# returned None.

# int dunders
print((5).__add__(3))
print((10).__sub__(4))
print((6).__mul__(7))
print((10).__floordiv__(3))
print((7).__mod__(2))
print((2).__pow__(10))
print((10).__truediv__(4))
print((12).__and__(10))
print((12).__or__(10))
print((12).__xor__(10))
print((-7).__neg__())
print((5).__pos__())
print((5).__invert__())
print((-3).__abs__())

# int → float/int conversions via dunder
print((42).__int__())
print((42).__float__())
print((0).__bool__())
print((5).__bool__())

# Float dunders
print((3.14).__floor__())
print((3.14).__ceil__())
print((3.7).__trunc__())
print((-3.0).__trunc__())
print((3.0).__int__())
print((5).__float__())
print((3.14).__neg__())

# Unbound-method form (int as a type)
print(int.__add__(10, 5))
print(int.__mul__(3, 4))
