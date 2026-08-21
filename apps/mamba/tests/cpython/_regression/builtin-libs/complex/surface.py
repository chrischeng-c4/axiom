# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/complex: surface probes (CPython 3.12 oracle)."""

# Probes for the documented complex API surface. Each `assert` verifies
# that an attribute / method exists and is callable.

z = 3 + 4j

# Type identity and literal-construction surface.
assert type(z) is complex
assert type(1j) is complex
assert isinstance(z, complex)

# Instance data attributes: .real and .imag are floats.
assert hasattr(z, "real")
assert hasattr(z, "imag")
assert isinstance(z.real, float)
assert isinstance(z.imag, float)

# Documented methods.
assert hasattr(z, "conjugate")
assert callable(z.conjugate)
assert hasattr(z, "__getnewargs__")

# Pickle/copy surface: __getnewargs__ returns the (real, imag) pair.
assert z.__getnewargs__() == (3.0, 4.0)

# Dunder operator surface lives on the type.
for name in ("__add__", "__sub__", "__mul__", "__truediv__",
             "__neg__", "__abs__", "__eq__", "__hash__"):
    assert hasattr(complex, name), name

print("surface OK")
