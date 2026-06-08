"""Surface contract for builtins.all.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "all"), "builtins.all missing"
assert builtins.all is all, f"builtins.all is all divergence"
assert callable(builtins.all), f"builtins.all not callable"
_t = type(builtins.all).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.all).__name__ = {_t!r}, expected 'builtin_function_or_method'"
assert builtins.all.__name__ == "all", \
    f"builtins.all.__name__ = {builtins.all.__name__!r}"
assert builtins.all.__module__ == "builtins", \
    f"builtins.all.__module__ = {builtins.all.__module__!r}"
assert isinstance(builtins.all.__doc__, str) and len(builtins.all.__doc__) > 0

print("surface OK")
