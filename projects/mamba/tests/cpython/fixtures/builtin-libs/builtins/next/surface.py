"""Surface contract for builtins.next.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "next"), "builtins.next missing"
assert builtins.next is next, f"builtins.next is next divergence"
assert callable(builtins.next), f"builtins.next not callable"
_t = type(builtins.next).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.next).__name__ = {_t!r}, expected 'builtin_function_or_method'"
assert builtins.next.__name__ == "next", \
    f"builtins.next.__name__ = {builtins.next.__name__!r}"
assert builtins.next.__module__ == "builtins", \
    f"builtins.next.__module__ = {builtins.next.__module__!r}"
assert isinstance(builtins.next.__doc__, str) and len(builtins.next.__doc__) > 0

print("surface OK")
