# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/mro_super: language-area surface probes (CPython 3.12 oracle)."""

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
assert hasattr(builtins, "super")
assert hasattr(builtins, "classmethod")
assert hasattr(builtins, "staticmethod")
assert hasattr(builtins, "property")

# `super` is a regular type whose instances expose __self__ / __self_class__
# / __thisclass__ once bound to an object.
assert isinstance(super, type)
assert hasattr(super, "__thisclass__")
assert hasattr(super, "__self__")
assert hasattr(super, "__self_class__")

# Every new-style class exposes its method resolution order machinery.
assert hasattr(object, "__mro__")
assert hasattr(object, "mro")
assert hasattr(type, "mro")

print("surface OK")
