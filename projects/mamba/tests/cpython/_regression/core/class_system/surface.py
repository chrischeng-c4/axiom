# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/class_system: language-area surface probes (CPython 3.12 oracle)."""

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
assert hasattr(builtins, "property")
assert hasattr(builtins, "classmethod")
assert hasattr(builtins, "staticmethod")
assert hasattr(builtins, "getattr")
assert hasattr(builtins, "setattr")
assert hasattr(builtins, "delattr")
assert hasattr(builtins, "hasattr")

# object is the root of every type's MRO.
assert object in int.__mro__
assert object in str.__mro__
assert type(object) is type
assert type(type) is type

# Class-machinery attributes exposed on a plain user class.
class _Probe:
    pass
for _attr in ("__name__", "__mro__", "__bases__", "__dict__", "__module__"):
    assert hasattr(_Probe, _attr), _attr
assert _Probe.__name__ == "_Probe"
assert _Probe.__bases__ == (object,)

print("surface OK")
