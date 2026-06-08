"""Surface contract for builtins.any.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "any"), "builtins.any missing"
assert builtins.any is any, f"builtins.any is any divergence"
assert callable(builtins.any), f"builtins.any not callable"
_t = type(builtins.any).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.any).__name__ = {_t!r}, expected 'builtin_function_or_method'"
assert builtins.any.__name__ == "any", \
    f"builtins.any.__name__ = {builtins.any.__name__!r}"
assert builtins.any.__module__ == "builtins", \
    f"builtins.any.__module__ = {builtins.any.__module__!r}"
assert isinstance(builtins.any.__doc__, str) and len(builtins.any.__doc__) > 0

print("surface OK")
