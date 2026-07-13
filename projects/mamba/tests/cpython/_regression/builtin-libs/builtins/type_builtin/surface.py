"""Surface contract for builtins.type.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, callable,
one-arg and three-arg forms, self-referential metaclass.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "type"), "builtins.type missing"
assert builtins.type is type, "builtins.type is type divergence"
assert callable(builtins.type), "builtins.type not callable"

# type is a type — and type(type) is type
assert type(builtins.type).__name__ == "type", \
    f"type(builtins.type).__name__ = {type(builtins.type).__name__!r}"
assert type(type) is type, "type(type) is type failed"

assert builtins.type.__name__ == "type", \
    f"builtins.type.__name__ = {builtins.type.__name__!r}"

# One-arg form: type(obj) returns the type of obj
assert type(42) is int, f"type(42) = {type(42).__name__!r}"
assert type(3.14) is float, f"type(3.14) = {type(3.14).__name__!r}"
assert type("x") is str, f"type('x') = {type('x').__name__!r}"
assert type([]) is list, f"type([]) = {type([]).__name__!r}"
assert type({}) is dict, f"type({{}}) = {type({}).__name__!r}"
assert type(()) is tuple, f"type(()) = {type(()).__name__!r}"
assert type(True) is bool, f"type(True) = {type(True).__name__!r}"

# Three-arg form: type(name, bases, dict) creates a new type
MyClass = type("MyClass", (object,), {"x": 42})
assert MyClass.__name__ == "MyClass", f"MyClass.__name__ = {MyClass.__name__!r}"
assert MyClass.x == 42, f"MyClass.x = {MyClass.x!r}"
assert isinstance(MyClass(), MyClass), "MyClass() not isinstance"

# type.__doc__ exists
assert isinstance(builtins.type.__doc__, str) and len(builtins.type.__doc__) > 0, \
    "builtins.type.__doc__ missing or empty"

print("surface OK")
