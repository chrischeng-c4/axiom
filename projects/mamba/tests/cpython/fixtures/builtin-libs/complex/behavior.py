# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/complex: core behavior asserts (CPython 3.12 oracle)."""

# Real / imag parts are always floats.
z = 1 + 2j
assert z.real == 1.0 and z.imag == 2.0
assert (1 - 2j).imag == -2.0

# Core binary arithmetic.
assert (1 + 2j) + (3 + 4j) == 4 + 6j
assert (5 + 6j) - (1 + 2j) == 4 + 4j
assert (1 + 2j) * (3 + 4j) == -5 + 10j
assert (1 + 2j) / (1 + 2j) == 1 + 0j

# Mixed int / float operands promote to complex.
assert 1j + 1 == complex(1, 1)
assert 1j - 1 == complex(-1, 1)
assert 1j * 20 == complex(0, 20)
assert 1j * -1 == complex(0, -1)

# Unary negation and conjugate.
assert -(1 + 6j) == -1 - 6j
assert (3 + 4j).conjugate() == 3 - 4j

# abs() is the Euclidean magnitude.
assert abs(3 + 4j) == 5.0
assert abs(complex(-3, -4)) == 5.0

# Truthiness: only 0+0j is falsy.
assert bool(1j) is True
assert bool(complex(0.0, 0.0)) is False
assert bool(complex(1e-300, 0.0)) is True

# A real-valued complex hashes like the equal int/float.
assert hash(complex(7, 0)) == hash(7)
assert hash(complex(0.5, 0.0)) == hash(0.5)

# __getnewargs__ exposes the (real, imag) pair as floats.
assert (1 - 2j).__getnewargs__() == (1.0, -2.0)
assert (-0j).__getnewargs__() == (0.0, -0.0)

print("behavior OK")
