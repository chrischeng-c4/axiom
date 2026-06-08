"""Surface contract for builtins.dict.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, class membership,
subclass of object, key dict methods present.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "dict"), "builtins.dict missing"
assert builtins.dict is dict, "builtins.dict is dict divergence"
assert callable(builtins.dict), "builtins.dict not callable"

# dict is a class (type)
assert type(builtins.dict).__name__ == "type", \
    f"type(builtins.dict).__name__ = {type(builtins.dict).__name__!r}"
assert issubclass(builtins.dict, object), "dict is not a subclass of object"

assert builtins.dict.__name__ == "dict", \
    f"builtins.dict.__name__ = {builtins.dict.__name__!r}"

# dict instances
assert isinstance({}, dict), "isinstance({}, dict) failed"
assert isinstance({"a": 1}, dict), "isinstance({'a':1}, dict) failed"

# Key dict methods present
for _meth in ("keys", "values", "items", "get", "setdefault", "update",
              "pop", "popitem", "clear", "copy", "fromkeys"):
    assert hasattr(dict, _meth), f"dict.{_meth} missing"
    assert callable(getattr(dict, _meth)), f"dict.{_meth} not callable"

# dict.__doc__ exists
assert isinstance(builtins.dict.__doc__, str) and len(builtins.dict.__doc__) > 0, \
    "builtins.dict.__doc__ missing or empty"

print("surface OK")
