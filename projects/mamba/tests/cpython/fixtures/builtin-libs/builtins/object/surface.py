"""Surface contract for builtins.object.

# type-regime: monomorphic

Probes: name presence, is the base type, __name__, callable, subclass of
nothing (it's the root), key dunder methods present.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "object"), "builtins.object missing"
assert builtins.object is object, "builtins.object is object divergence"
assert callable(builtins.object), "builtins.object not callable"

# object is a type
assert type(builtins.object).__name__ == "type", \
    f"type(builtins.object).__name__ = {type(builtins.object).__name__!r}"
assert builtins.object.__name__ == "object", \
    f"builtins.object.__name__ = {builtins.object.__name__!r}"

# object is the base of all types
assert issubclass(int, object), "int not subclass of object"
assert issubclass(str, object), "str not subclass of object"
assert issubclass(list, object), "list not subclass of object"

# object instances
o = object()
assert isinstance(o, object), "isinstance(object(), object) failed"

# Key dunder methods on object
for _meth in ("__init__", "__new__", "__repr__", "__str__", "__eq__",
              "__ne__", "__hash__", "__setattr__", "__getattribute__",
              "__delattr__", "__class__", "__doc__"):
    assert hasattr(object, _meth), f"object.{_meth} missing"

# object.__doc__ exists
assert isinstance(builtins.object.__doc__, str) and len(builtins.object.__doc__) > 0, \
    "builtins.object.__doc__ missing"

print("surface OK")
