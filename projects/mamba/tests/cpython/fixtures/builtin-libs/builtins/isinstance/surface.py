"""Surface contract for builtins.isinstance.

# type-regime: monomorphic

Probes: name presence, callable, returns bool, two-arg form,
tuple-of-types form, subclass awareness.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "isinstance"), "builtins.isinstance missing"
assert builtins.isinstance is isinstance, "builtins.isinstance is isinstance divergence"
assert callable(builtins.isinstance), "builtins.isinstance not callable"

# isinstance returns bool
assert isinstance(isinstance(1, int), bool), "isinstance returns bool"

# Basic positive cases
assert isinstance(1, int) is True
assert isinstance(1.0, float) is True
assert isinstance("x", str) is True
assert isinstance([], list) is True
assert isinstance({}, dict) is True
assert isinstance(True, bool) is True
assert isinstance(True, int) is True  # bool is subtype of int

# Basic negative cases
assert isinstance(1, str) is False
assert isinstance("x", int) is False

# Tuple of types form
assert isinstance(1, (int, str)) is True
assert isinstance("x", (int, str)) is True
assert isinstance([], (int, str)) is False

# Wrong second arg raises TypeError
_raised = False
try:
    isinstance(1, 42)  # type: ignore[arg-type]
except TypeError:
    _raised = True
assert _raised, "isinstance(1, 42) did not raise TypeError"

print("surface OK")
