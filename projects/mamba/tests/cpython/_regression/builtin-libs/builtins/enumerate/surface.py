"""Surface contract for builtins.enumerate.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity (enumerate is a
class), __name__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "enumerate"), "builtins.enumerate missing"

assert builtins.enumerate is enumerate, \
    f"builtins.enumerate is enumerate divergence"

assert callable(builtins.enumerate), \
    f"builtins.enumerate not callable: type={type(builtins.enumerate).__name__}"

_t = type(builtins.enumerate).__name__
assert _t == "type", \
    f"type(builtins.enumerate).__name__ = {_t!r}, expected 'type'"

assert builtins.enumerate.__name__ == "enumerate", \
    f"builtins.enumerate.__name__ = {builtins.enumerate.__name__!r}, expected 'enumerate'"

assert isinstance(builtins.enumerate.__doc__, str) and len(builtins.enumerate.__doc__) > 0, \
    f"builtins.enumerate.__doc__ invalid: {builtins.enumerate.__doc__!r}"

print("surface OK")
