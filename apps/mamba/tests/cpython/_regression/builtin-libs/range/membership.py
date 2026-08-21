# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""range: membership (`in`) semantics, including stride and type rules."""


# An object whose __eq__ always returns True is found via linear scan.
class AlwaysEqual:
    def __eq__(self, other):
        return True

    def __hash__(self):
        return 1


# Forward unit range: membership is bounds-checked.
r = range(10)
assert 0 in r
assert 5 in r
assert -1 not in r
assert 10 not in r
assert "" not in r

# Reverse unit range covers the same elements.
r = range(9, -1, -1)
assert 0 in r
assert 9 in r
assert -1 not in r
assert 10 not in r

# Strided range: membership respects the step, not just the bounds.
r = range(0, 10, 2)
assert 0 in r
assert 2 in r
assert 8 in r
assert 1 not in r
assert 5 not in r
assert 10 not in r

# Reverse strided range.
r = range(9, -1, -2)
assert 9 in r
assert 1 in r
assert 0 not in r
assert 8 not in r

# Larger stride limits, both directions.
r = range(0, 101, 2)
assert 100 in r
assert 101 not in r
r = range(0, -20, -2)
assert -18 in r
assert -19 not in r
assert -20 not in r

# Empty ranges contain nothing.
assert 0 not in range(0)
assert 0 not in range(0, -10)
assert -1 not in range(0, -10)

# Numeric coercion: float, bool, and complex values that equal an element
# are considered members; non-equal floats are not.
assert 5.0 in range(10)
assert 5.1 not in range(10)
assert True in range(3)
assert (1 + 0j) in range(3)

# A non-int object with __index__ is NOT a member unless it compares equal;
# int() of it can be.
class HasIndex:
    def __int__(self):
        return 1

    def __index__(self):
        return 1


assert HasIndex() not in range(3)
assert int(HasIndex()) in range(3)

# An always-equal object short-circuits the scan and is found.
assert AlwaysEqual() in range(10)

# An int subclass whose __eq__ is always True is found even past the bound.
class EqInt(int):
    def __eq__(self, other):
        return True

    def __hash__(self):
        return 0


assert EqInt(11) in range(10)

print("membership OK")
