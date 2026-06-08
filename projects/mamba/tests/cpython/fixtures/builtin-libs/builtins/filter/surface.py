"""Surface contract for builtins.filter.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity (filter is a
class), __name__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "filter"), "builtins.filter missing"

assert builtins.filter is filter, \
    f"builtins.filter is filter divergence"

assert callable(builtins.filter), \
    f"builtins.filter not callable: type={type(builtins.filter).__name__}"

_t = type(builtins.filter).__name__
assert _t == "type", \
    f"type(builtins.filter).__name__ = {_t!r}, expected 'type'"

assert builtins.filter.__name__ == "filter", \
    f"builtins.filter.__name__ = {builtins.filter.__name__!r}, expected 'filter'"

assert isinstance(builtins.filter.__doc__, str) and len(builtins.filter.__doc__) > 0, \
    f"builtins.filter.__doc__ invalid: {builtins.filter.__doc__!r}"

print("surface OK")
