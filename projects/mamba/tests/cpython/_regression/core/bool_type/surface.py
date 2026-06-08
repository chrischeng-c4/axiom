# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/bool_type: language-area surface probes (CPython 3.12 oracle)."""

# Core language constructs always available.
import builtins
assert hasattr(builtins, "object")
assert hasattr(builtins, "type")
assert hasattr(builtins, "list")
assert hasattr(builtins, "dict")
assert hasattr(builtins, "set")
assert hasattr(builtins, "tuple")
assert hasattr(builtins, "str")
assert hasattr(builtins, "bytes")
assert hasattr(builtins, "int")
assert hasattr(builtins, "float")
assert hasattr(builtins, "complex")
assert hasattr(builtins, "bool")
assert hasattr(builtins, "isinstance")
assert hasattr(builtins, "issubclass")
assert hasattr(builtins, "callable")

# bool is a real type that subclasses int.
assert isinstance(bool, type)
assert bool.__bases__ == (int,)
assert issubclass(bool, int)

# bool carries int's constructor/serialisation hooks and numeric attributes.
assert hasattr(bool, "__new__")
assert hasattr(bool, "from_bytes")
assert hasattr(bool, "to_bytes")
assert hasattr(True, "real") and hasattr(True, "imag")
assert hasattr(True, "__bool__") and hasattr(True, "__int__")

print("surface OK")
