"""Surface contract for builtins.set.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, class membership,
subclass of object, key set methods present.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "set"), "builtins.set missing"
assert builtins.set is set, "builtins.set is set divergence"
assert callable(builtins.set), "builtins.set not callable"

# set is a class (type)
assert type(builtins.set).__name__ == "type", \
    f"type(builtins.set).__name__ = {type(builtins.set).__name__!r}"
assert issubclass(builtins.set, object), "set is not a subclass of object"

assert builtins.set.__name__ == "set", \
    f"builtins.set.__name__ = {builtins.set.__name__!r}"

# set instances
assert isinstance(set(), set), "isinstance(set(), set) failed"
assert isinstance({1, 2, 3}, set), "isinstance({1,2,3}, set) failed"

# Key set methods present
for _meth in ("add", "discard", "remove", "pop", "clear", "copy",
              "union", "intersection", "difference", "symmetric_difference",
              "issubset", "issuperset", "isdisjoint",
              "update", "intersection_update", "difference_update",
              "symmetric_difference_update"):
    assert hasattr(set, _meth), f"set.{_meth} missing"
    assert callable(getattr(set, _meth)), f"set.{_meth} not callable"

# set.__doc__ exists
assert isinstance(builtins.set.__doc__, str) and len(builtins.set.__doc__) > 0, \
    "builtins.set.__doc__ missing or empty"

print("surface OK")
