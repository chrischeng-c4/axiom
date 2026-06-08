"""Surface contract for builtins.iter.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "iter"), "builtins.iter missing"
assert builtins.iter is iter, f"builtins.iter is iter divergence"
assert callable(builtins.iter), f"builtins.iter not callable"
_t = type(builtins.iter).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.iter).__name__ = {_t!r}, expected 'builtin_function_or_method'"
assert builtins.iter.__name__ == "iter", \
    f"builtins.iter.__name__ = {builtins.iter.__name__!r}"
assert builtins.iter.__module__ == "builtins", \
    f"builtins.iter.__module__ = {builtins.iter.__module__!r}"
assert isinstance(builtins.iter.__doc__, str) and len(builtins.iter.__doc__) > 0

print("surface OK")
