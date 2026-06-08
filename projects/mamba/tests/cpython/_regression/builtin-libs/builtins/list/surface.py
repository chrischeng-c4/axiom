"""Surface contract for builtins.list.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, class membership,
subclass of object, key list methods present.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "list"), "builtins.list missing"
assert builtins.list is list, "builtins.list is list divergence"
assert callable(builtins.list), "builtins.list not callable"

# list is a class (type)
assert type(builtins.list).__name__ == "type", \
    f"type(builtins.list).__name__ = {type(builtins.list).__name__!r}"
assert issubclass(builtins.list, object), "list is not a subclass of object"

assert builtins.list.__name__ == "list", \
    f"builtins.list.__name__ = {builtins.list.__name__!r}"

# list instances
assert isinstance([], list), "isinstance([], list) failed"
assert isinstance([1, 2, 3], list), "isinstance([1,2,3], list) failed"

# Key list methods present
for _meth in ("append", "extend", "insert", "remove", "pop", "clear",
              "index", "count", "sort", "reverse", "copy"):
    assert hasattr(list, _meth), f"list.{_meth} missing"
    assert callable(getattr(list, _meth)), f"list.{_meth} not callable"

# list.__doc__ exists
assert isinstance(builtins.list.__doc__, str) and len(builtins.list.__doc__) > 0, \
    "builtins.list.__doc__ missing or empty"

print("surface OK")
