"""Surface contract for builtins.format.

# type-regime: monomorphic

Probes: name presence, callable, returns str, one-arg and two-arg forms,
delegates to __format__.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "format"), "builtins.format missing"
assert builtins.format is format, "builtins.format is format divergence"
assert callable(builtins.format), "builtins.format not callable"

# format returns str
assert isinstance(format(42), str), "format(42) not str"
assert isinstance(format(3.14, ".2f"), str), "format(3.14, '.2f') not str"

# one-arg form: format(value) == str(value) for basic types
assert format(42) == "42", f"format(42) = {format(42)!r}"
assert format(True) == "True", f"format(True) = {format(True)!r}"
assert format(None) == "None", f"format(None) = {format(None)!r}"

# two-arg form: format(value, spec)
assert format(42, "d") == "42", f"format(42,'d') = {format(42,'d')!r}"
assert format(42, "08b") == "00101010", f"format(42,'08b') = {format(42,'08b')!r}"

print("surface OK")
