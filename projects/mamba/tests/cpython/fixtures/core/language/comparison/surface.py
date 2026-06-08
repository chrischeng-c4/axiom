"""Surface contract for language comparison operators.

# type-regime: monomorphic

Probes: ==, !=, <, <=, >, >=, is, is not, in, not in — all return bool,
chained comparisons work, identity vs equality distinction.
CPython 3.12 is the oracle.
"""

# All comparison operators return bool
assert isinstance(1 == 1, bool)
assert isinstance(1 != 2, bool)
assert isinstance(1 < 2, bool)
assert isinstance(1 <= 1, bool)
assert isinstance(2 > 1, bool)
assert isinstance(2 >= 2, bool)

_a = [1, 2]
assert isinstance(_a is _a, bool)
assert isinstance(_a is not [], bool)
assert isinstance(1 in [1, 2], bool)
assert isinstance(3 not in [1, 2], bool)

# Basic comparisons
assert (1 == 1) == True
assert (1 == 2) == False
assert (1 != 2) == True
assert (1 < 2) == True
assert (1 <= 1) == True
assert (2 > 1) == True
assert (2 >= 2) == True

# Chained comparisons
assert (1 < 2 < 3) == True
assert (1 < 2 > 0) == True
assert not (1 < 2 < 2)

# Identity vs equality
a = [1, 2]
b = [1, 2]
assert a == b           # equal content
assert a is not b       # different objects

c = a
assert c is a           # same object
assert c == a

print("surface OK")
