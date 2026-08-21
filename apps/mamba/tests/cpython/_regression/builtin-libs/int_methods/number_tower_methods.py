# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# int / float number-tower method coverage:
#   int.is_integer(), int.as_integer_ratio(), float.__round__()
# (Property-style access — (5).numerator / (5).real / etc. — is currently
# JIT-specialised in a way that bypasses mb_getattr's primitive arms; the
# method/dunder forms below are the cases we lock in here.)

# int.is_integer — always True for any int in CPython.
print((0).is_integer())
print((5).is_integer())
print((-3).is_integer())
print((100).is_integer())

# int.as_integer_ratio — (n, 1).
print((0).as_integer_ratio())
print((5).as_integer_ratio())
print((-3).as_integer_ratio())
print((100).as_integer_ratio())

# float.__round__() — banker's rounding (half to even), returns int.
print((1.5).__round__())
print((2.5).__round__())
print((-1.5).__round__())
print((-2.5).__round__())
print((0.5).__round__())
print((1.4).__round__())
print((1.6).__round__())
print((3.7).__round__())

# float.__round__(ndigits) — returns float.
print((3.14159).__round__(2))
print((-3.14159).__round__(2))
print((1.0).__round__(2))
print((100.5).__round__(0))
