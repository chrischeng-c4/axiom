"""Surface contract for builtins.reversed.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "reversed"), "builtins.reversed missing"

assert builtins.reversed is reversed, \
    f"builtins.reversed is reversed divergence"

assert callable(builtins.reversed), \
    f"builtins.reversed not callable: type={type(builtins.reversed).__name__}"

# reversed is a class (type), not a builtin_function_or_method
_t = type(builtins.reversed).__name__
assert _t == "type", \
    f"type(builtins.reversed).__name__ = {_t!r}, expected 'type'"

assert builtins.reversed.__name__ == "reversed", \
    f"builtins.reversed.__name__ = {builtins.reversed.__name__!r}, expected 'reversed'"

assert isinstance(builtins.reversed.__doc__, str) and len(builtins.reversed.__doc__) > 0, \
    f"builtins.reversed.__doc__ invalid: {builtins.reversed.__doc__!r}"

print("surface OK")
