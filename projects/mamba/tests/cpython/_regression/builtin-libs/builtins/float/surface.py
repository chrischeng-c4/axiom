"""Surface contract for builtins.float.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, class membership,
subclass of object, zero-arg default, special values.
CPython 3.12 is the oracle.
"""

import builtins
import math

assert hasattr(builtins, "float"), "builtins.float missing"
assert builtins.float is float, "builtins.float is float divergence"
assert callable(builtins.float), "builtins.float not callable"

# float is a class (type)
assert type(builtins.float).__name__ == "type", \
    f"type(builtins.float).__name__ = {type(builtins.float).__name__!r}"
assert issubclass(builtins.float, object), "float is not a subclass of object"

assert builtins.float.__name__ == "float", \
    f"builtins.float.__name__ = {builtins.float.__name__!r}"

# float instances
assert isinstance(0.0, float), "isinstance(0.0, float) failed"
assert isinstance(-1.5, float), "isinstance(-1.5, float) failed"
assert isinstance(3.14, float), "isinstance(3.14, float) failed"

# special float values via math
assert math.isinf(float("inf")), "float('inf') is not inf"
assert math.isinf(float("-inf")), "float('-inf') is not -inf"
assert math.isnan(float("nan")), "float('nan') is not nan"

# float.__doc__ exists
assert isinstance(builtins.float.__doc__, str) and len(builtins.float.__doc__) > 0, \
    "builtins.float.__doc__ missing or empty"

print("surface OK")
