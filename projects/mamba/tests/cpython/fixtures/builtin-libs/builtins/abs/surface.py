"""Surface contract for builtins.abs.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity, __name__,
__module__, __doc__. CPython 3.12 is the oracle.
"""

import builtins

# Name presence
assert hasattr(builtins, "abs"), "builtins.abs missing"

# Same identity via global lookup
assert builtins.abs is abs, \
    f"builtins.abs is abs divergence: {builtins.abs!r} vs {abs!r}"

# Callable
assert callable(builtins.abs), \
    f"builtins.abs not callable: type={type(builtins.abs).__name__}"

# Type identity
_t = type(builtins.abs).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.abs).__name__ = {_t!r}, expected 'builtin_function_or_method'"

# Documented name
assert builtins.abs.__name__ == "abs", \
    f"builtins.abs.__name__ = {builtins.abs.__name__!r}, expected 'abs'"

# Documented module
assert builtins.abs.__module__ == "builtins", \
    f"builtins.abs.__module__ = {builtins.abs.__module__!r}, expected 'builtins'"

# Non-empty docstring
assert isinstance(builtins.abs.__doc__, str) and len(builtins.abs.__doc__) > 0, \
    f"builtins.abs.__doc__ invalid: {builtins.abs.__doc__!r}"

print("surface OK")
