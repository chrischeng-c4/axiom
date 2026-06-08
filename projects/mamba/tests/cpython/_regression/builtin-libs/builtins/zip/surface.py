"""Surface contract for builtins.zip.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity (zip is a class),
__name__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "zip"), "builtins.zip missing"

assert builtins.zip is zip, \
    f"builtins.zip is zip divergence"

assert callable(builtins.zip), \
    f"builtins.zip not callable: type={type(builtins.zip).__name__}"

# zip is a type (class), not a builtin_function_or_method
_t = type(builtins.zip).__name__
assert _t == "type", \
    f"type(builtins.zip).__name__ = {_t!r}, expected 'type'"

assert builtins.zip.__name__ == "zip", \
    f"builtins.zip.__name__ = {builtins.zip.__name__!r}, expected 'zip'"

assert isinstance(builtins.zip.__doc__, str) and len(builtins.zip.__doc__) > 0, \
    f"builtins.zip.__doc__ invalid: {builtins.zip.__doc__!r}"

print("surface OK")
