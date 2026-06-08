# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/complex: string / numeric construction (CPython 3.12 oracle)."""

from math import copysign, isnan

INF = float("inf")

# Two-argument form and equivalent literal.
assert complex(1, 2) == 1 + 2j
assert complex(3) == 3 + 0j
assert complex() == 0j

# Plain string forms (no surrounding parens needed).
assert complex("1+2j") == 1 + 2j
assert complex("-3.5j") == complex(0.0, -3.5)
assert complex("(1+2j)") == 1 + 2j

# Magnitudes too large for a double overflow to infinity, not error.
assert complex("1e500") == complex(INF, 0.0)
assert complex("-1e500j") == complex(0.0, -INF)
assert complex("-1e500+1.8e308j") == complex(-INF, INF)

# Sign of NaN is preserved through string parsing.
assert isnan(complex("-nan").real)
assert copysign(1.0, complex("-nan").real) == -1.0
assert copysign(1.0, complex("-nanj").imag) == -1.0
assert copysign(1.0, complex("-nan-nanj").real) == -1.0
assert copysign(1.0, complex("-nan-nanj").imag) == -1.0

# Underscores group digits in numeric literals and string args alike;
# they must not change the value.
assert complex("1_000+2_000j") == complex(1000, 2000)
assert complex("1_000.5j") == complex(0.0, 1000.5)
assert 1_0 + 2_0j == complex(10, 20)

# Misplaced underscores are rejected.
for bad in ("1__0+0j", "1_+0j", "_1+0j", "1+_2j"):
    try:
        complex(bad)
        raise AssertionError("expected ValueError for %r" % bad)
    except ValueError:
        pass

print("constructor OK")
