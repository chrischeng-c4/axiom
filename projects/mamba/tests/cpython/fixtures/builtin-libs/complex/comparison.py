# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/complex: equality and ordering rules (CPython 3.12 oracle)."""

import operator

# Equality is structural across real and imaginary parts.
assert (1 + 1j) == (1 + 1j)
assert (1 + 1j) != (2 + 2j)
assert complex.__eq__(1 + 1j, 1 + 1j) is True
assert complex.__eq__(1 + 1j, 2 + 2j) is False
assert complex.__ne__(1 + 1j, 2 + 2j) is True

# A complex with zero imaginary part equals the matching real number;
# a non-zero imaginary part never does.
for i in range(1, 5):
    f = i / 10.0
    assert complex.__eq__(f + 0j, f) is True
    assert complex.__eq__(complex(f, f), f) is False

# Comparison against an unrelated / non-numeric type yields NotImplemented
# at the dunder level (so == falls back to identity / returns False).
assert complex.__eq__(1 + 1j, None) is NotImplemented
assert complex.__lt__(1 + 1j, None) is NotImplemented
assert (1 + 1j) != None
assert (1 + 1j) != "1+1j"

# An int too large to convert exactly still compares unequal, not error.
assert complex.__eq__(1 + 1j, 1 << 10000) is False

# Ordering is undefined: the dunders return NotImplemented and operator.*
# raises TypeError.
assert complex.__lt__(1 + 1j, 2 + 2j) is NotImplemented
assert complex.__le__(1 + 1j, 2 + 2j) is NotImplemented
assert complex.__gt__(1 + 1j, 2 + 2j) is NotImplemented
assert complex.__ge__(1 + 1j, 2 + 2j) is NotImplemented
for op in (operator.lt, operator.le, operator.gt, operator.ge):
    try:
        op(1 + 1j, 2 + 2j)
        raise AssertionError("expected TypeError from %r" % op)
    except TypeError:
        pass

# Equality is allowed across complex and real numbers.
assert operator.eq(1 + 1j, 2.0) is False
assert operator.eq(2 + 0j, 2.0) is True

print("comparison OK")
