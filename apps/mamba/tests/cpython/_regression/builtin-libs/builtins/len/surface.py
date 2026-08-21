"""Surface contract for builtins.len.

# type-regime: monomorphic

Probes: name presence on the builtins module, callable shape, no
default identity on a built-in slot wrapper. CPython 3.12 is the
oracle — every assertion is what CPython produces.
"""

import builtins

# Name presence on builtins module
assert hasattr(builtins, "len"), "builtins.len missing"

# Same identity via global name lookup vs attribute on builtins
assert builtins.len is len, \
    f"builtins.len is len divergence: builtins.len={builtins.len!r}, len={len!r}"

# Callable shape
assert callable(builtins.len), \
    f"builtins.len not callable: type={type(builtins.len).__name__}"

# Type identity: len is a builtin function, not a Python function
_t = type(builtins.len).__name__
assert _t == "builtin_function_or_method", \
    f"type(builtins.len).__name__ = {_t!r}, expected 'builtin_function_or_method'"

# Documented name
assert builtins.len.__name__ == "len", \
    f"builtins.len.__name__ = {builtins.len.__name__!r}, expected 'len'"

# Documented module
assert builtins.len.__module__ == "builtins", \
    f"builtins.len.__module__ = {builtins.len.__module__!r}, expected 'builtins'"

# Has __doc__ (non-empty docstring)
assert isinstance(builtins.len.__doc__, str), \
    f"builtins.len.__doc__ type = {type(builtins.len.__doc__).__name__}, expected str"
assert len(builtins.len.__doc__) > 0, \
    f"builtins.len.__doc__ is empty"

print("surface OK")
