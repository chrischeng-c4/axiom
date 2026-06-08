"""Surface contract for builtins.sorted.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "sorted"), "builtins.sorted missing"

assert builtins.sorted is sorted, \
    f"builtins.sorted is sorted divergence: {builtins.sorted!r} vs {sorted!r}"

assert callable(builtins.sorted), \
    f"builtins.sorted not callable: type={type(builtins.sorted).__name__}"

_t = type(builtins.sorted).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.sorted).__name__ = {_t!r}, expected 'builtin_function_or_method'"

assert builtins.sorted.__name__ == "sorted", \
    f"builtins.sorted.__name__ = {builtins.sorted.__name__!r}, expected 'sorted'"

assert builtins.sorted.__module__ == "builtins", \
    f"builtins.sorted.__module__ = {builtins.sorted.__module__!r}, expected 'builtins'"

assert isinstance(builtins.sorted.__doc__, str) and len(builtins.sorted.__doc__) > 0, \
    f"builtins.sorted.__doc__ invalid: {builtins.sorted.__doc__!r}"

print("surface OK")
