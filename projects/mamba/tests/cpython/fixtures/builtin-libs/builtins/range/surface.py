"""Surface contract for builtins.range.

# type-regime: monomorphic

Probes: name presence, callable shape, type identity (range is a
class), __name__, __doc__, and core attributes of range instances.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "range"), "builtins.range missing"

assert builtins.range is range, \
    f"builtins.range is range divergence"

assert callable(builtins.range), \
    f"builtins.range not callable: type={type(builtins.range).__name__}"

_t = type(builtins.range).__name__
assert _t == "type", \
    f"type(builtins.range).__name__ = {_t!r}, expected 'type'"

assert builtins.range.__name__ == "range", \
    f"builtins.range.__name__ = {builtins.range.__name__!r}, expected 'range'"

assert isinstance(builtins.range.__doc__, str) and len(builtins.range.__doc__) > 0, \
    f"builtins.range.__doc__ invalid: {builtins.range.__doc__!r}"

# range instance attributes
_r = range(2, 10, 3)
assert _r.start == 2, f"range.start = {_r.start!r}, expected 2"
assert _r.stop == 10, f"range.stop = {_r.stop!r}, expected 10"
assert _r.step == 3, f"range.step = {_r.step!r}, expected 3"

print("surface OK")
