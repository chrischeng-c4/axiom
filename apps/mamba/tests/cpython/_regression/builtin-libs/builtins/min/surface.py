"""Surface contract for builtins.min.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "min"), "builtins.min missing"

assert builtins.min is min, \
    f"builtins.min is min divergence: {builtins.min!r} vs {min!r}"

assert callable(builtins.min), \
    f"builtins.min not callable: type={type(builtins.min).__name__}"

_t = type(builtins.min).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.min).__name__ = {_t!r}, expected 'builtin_function_or_method'"

assert builtins.min.__name__ == "min", \
    f"builtins.min.__name__ = {builtins.min.__name__!r}, expected 'min'"

assert builtins.min.__module__ == "builtins", \
    f"builtins.min.__module__ = {builtins.min.__module__!r}, expected 'builtins'"

assert isinstance(builtins.min.__doc__, str) and len(builtins.min.__doc__) > 0, \
    f"builtins.min.__doc__ invalid: {builtins.min.__doc__!r}"

print("surface OK")
