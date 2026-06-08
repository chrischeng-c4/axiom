# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/float_methods: surface probes (CPython 3.12 oracle)."""

# The documented float API surface: instance methods, classmethods, and
# the numeric dunders every float exposes. Each assert verifies a name is
# present and callable / of the documented type.

# Instance methods.
assert callable(float.as_integer_ratio)
assert callable(float.is_integer)
assert callable(float.hex)
assert callable(float.conjugate)
assert callable(float.__round__)
assert callable(float.__ceil__)
assert callable(float.__floor__)
assert callable(float.__trunc__)

# fromhex is a classmethod constructor.
assert callable(float.fromhex)
assert float.fromhex("0x1.8p0") == 1.5

# Read-only numeric "complex-like" attributes (real/imag; conjugate is a no-op).
assert (3.5).real == 3.5
assert (3.5).imag == 0.0
assert (3.5).conjugate() == 3.5

# float() with no argument yields 0.0.
assert float() == 0.0

# float subclasses int's numeric protocol but is its own type.
assert isinstance(1.0, float)
assert not isinstance(1.0, int)
assert type(1.0) is float

print("surface OK")
