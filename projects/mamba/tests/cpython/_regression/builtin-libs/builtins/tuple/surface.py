"""Surface contract for builtins.tuple.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, class membership,
subclass of object, key tuple methods present, immutability.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "tuple"), "builtins.tuple missing"
assert builtins.tuple is tuple, "builtins.tuple is tuple divergence"
assert callable(builtins.tuple), "builtins.tuple not callable"

# tuple is a class (type)
assert type(builtins.tuple).__name__ == "type", \
    f"type(builtins.tuple).__name__ = {type(builtins.tuple).__name__!r}"
assert issubclass(builtins.tuple, object), "tuple is not a subclass of object"

assert builtins.tuple.__name__ == "tuple", \
    f"builtins.tuple.__name__ = {builtins.tuple.__name__!r}"

# tuple instances
assert isinstance((), tuple), "isinstance((), tuple) failed"
assert isinstance((1, 2), tuple), "isinstance((1,2), tuple) failed"

# Key tuple methods present
for _meth in ("count", "index"):
    assert hasattr(tuple, _meth), f"tuple.{_meth} missing"
    assert callable(getattr(tuple, _meth)), f"tuple.{_meth} not callable"

# tuple is immutable
_raised = False
try:
    t = (1, 2, 3)
    t[0] = 99  # type: ignore[index]
except TypeError:
    _raised = True
assert _raised, "tuple assignment did not raise TypeError"

# tuple.__doc__ exists
assert isinstance(builtins.tuple.__doc__, str) and len(builtins.tuple.__doc__) > 0, \
    "builtins.tuple.__doc__ missing or empty"

print("surface OK")
