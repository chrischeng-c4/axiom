"""Surface contract for builtins.sum.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "sum"), "builtins.sum missing"

assert builtins.sum is sum, \
    f"builtins.sum is sum divergence: {builtins.sum!r} vs {sum!r}"

assert callable(builtins.sum), \
    f"builtins.sum not callable: type={type(builtins.sum).__name__}"

_t = type(builtins.sum).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.sum).__name__ = {_t!r}, expected 'builtin_function_or_method'"

assert builtins.sum.__name__ == "sum", \
    f"builtins.sum.__name__ = {builtins.sum.__name__!r}, expected 'sum'"

assert builtins.sum.__module__ == "builtins", \
    f"builtins.sum.__module__ = {builtins.sum.__module__!r}, expected 'builtins'"

assert isinstance(builtins.sum.__doc__, str) and len(builtins.sum.__doc__) > 0, \
    f"builtins.sum.__doc__ invalid: {builtins.sum.__doc__!r}"

print("surface OK")
