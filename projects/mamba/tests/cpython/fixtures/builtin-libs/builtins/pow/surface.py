"""Surface contract for builtins.pow.

# type-regime: monomorphic

Probes: name presence, callable, two-arg and three-arg forms.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "pow"), "builtins.pow missing"
assert builtins.pow is pow, "builtins.pow is pow divergence"
assert callable(builtins.pow), "builtins.pow not callable"

# two-arg form: pow(base, exp)
assert pow(2, 10) == 1024, f"pow(2,10) = {pow(2,10)!r}"
assert isinstance(pow(2, 10), int), "pow(int,int) returns int"
assert isinstance(pow(2.0, 3), float), "pow(float,int) returns float"

# three-arg form: pow(base, exp, mod)
assert pow(2, 10, 1000) == 24, f"pow(2,10,1000) = {pow(2,10,1000)!r}"
assert isinstance(pow(2, 10, 1000), int), "pow(int,int,int) returns int"

print("surface OK")
