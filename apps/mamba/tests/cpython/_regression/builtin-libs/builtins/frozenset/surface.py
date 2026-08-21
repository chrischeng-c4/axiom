"""Surface contract for builtins.frozenset.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, class membership,
subclass of object, key frozenset methods present, hashability.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "frozenset"), "builtins.frozenset missing"
assert builtins.frozenset is frozenset, "builtins.frozenset is frozenset divergence"
assert callable(builtins.frozenset), "builtins.frozenset not callable"

# frozenset is a class (type)
assert type(builtins.frozenset).__name__ == "type", \
    f"type(builtins.frozenset).__name__ = {type(builtins.frozenset).__name__!r}"
assert issubclass(builtins.frozenset, object), "frozenset is not a subclass of object"

assert builtins.frozenset.__name__ == "frozenset", \
    f"builtins.frozenset.__name__ = {builtins.frozenset.__name__!r}"

# frozenset instances
assert isinstance(frozenset(), frozenset), "isinstance(frozenset(), frozenset) failed"
assert isinstance(frozenset([1, 2, 3]), frozenset), "isinstance failed"

# frozenset is hashable (unlike set)
h = hash(frozenset([1, 2, 3]))
assert isinstance(h, int), f"hash(frozenset) not int: {h!r}"

# Key frozenset methods present (read-only, no add/remove/update)
for _meth in ("union", "intersection", "difference", "symmetric_difference",
              "issubset", "issuperset", "isdisjoint", "copy"):
    assert hasattr(frozenset, _meth), f"frozenset.{_meth} missing"
    assert callable(getattr(frozenset, _meth)), f"frozenset.{_meth} not callable"

# frozenset is immutable — no add/remove
assert not hasattr(frozenset, "add"), "frozenset should not have add"
assert not hasattr(frozenset, "remove"), "frozenset should not have remove"

# frozenset.__doc__ exists
assert isinstance(builtins.frozenset.__doc__, str) and len(builtins.frozenset.__doc__) > 0, \
    "builtins.frozenset.__doc__ missing or empty"

print("surface OK")
