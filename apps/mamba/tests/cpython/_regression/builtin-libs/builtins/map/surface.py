"""Surface contract for builtins.map.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity (map is a class),
__name__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "map"), "builtins.map missing"

assert builtins.map is map, \
    f"builtins.map is map divergence"

assert callable(builtins.map), \
    f"builtins.map not callable: type={type(builtins.map).__name__}"

_t = type(builtins.map).__name__
assert _t == "type", \
    f"type(builtins.map).__name__ = {_t!r}, expected 'type'"

assert builtins.map.__name__ == "map", \
    f"builtins.map.__name__ = {builtins.map.__name__!r}, expected 'map'"

assert isinstance(builtins.map.__doc__, str) and len(builtins.map.__doc__) > 0, \
    f"builtins.map.__doc__ invalid: {builtins.map.__doc__!r}"

print("surface OK")
