"""Surface contract for builtins.max.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "max"), "builtins.max missing"

assert builtins.max is max, \
    f"builtins.max is max divergence: {builtins.max!r} vs {max!r}"

assert callable(builtins.max), \
    f"builtins.max not callable: type={type(builtins.max).__name__}"

_t = type(builtins.max).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.max).__name__ = {_t!r}, expected 'builtin_function_or_method'"

assert builtins.max.__name__ == "max", \
    f"builtins.max.__name__ = {builtins.max.__name__!r}, expected 'max'"

assert builtins.max.__module__ == "builtins", \
    f"builtins.max.__module__ = {builtins.max.__module__!r}, expected 'builtins'"

assert isinstance(builtins.max.__doc__, str) and len(builtins.max.__doc__) > 0, \
    f"builtins.max.__doc__ invalid: {builtins.max.__doc__!r}"

print("surface OK")
