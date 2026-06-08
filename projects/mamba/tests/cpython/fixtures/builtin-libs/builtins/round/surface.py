"""Surface contract for builtins.round.

# type-regime: monomorphic

Probes: name presence, callable, one-arg and two-arg forms,
banker's rounding (round-half-to-even).
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "round"), "builtins.round missing"
assert builtins.round is round, "builtins.round is round divergence"
assert callable(builtins.round), "builtins.round not callable"

# round(float) → int
assert isinstance(round(3.5), int), "round(float) returns int"
assert isinstance(round(3), int), "round(int) returns int"

# round(float, ndigits) → float
assert isinstance(round(3.14, 1), float), "round(float, ndigits) returns float"

# Banker's rounding (round half to even)
assert round(0.5) == 0, f"round(0.5) = {round(0.5)!r}"   # CPython: 0 (even)
assert round(1.5) == 2, f"round(1.5) = {round(1.5)!r}"   # CPython: 2 (even)
assert round(2.5) == 2, f"round(2.5) = {round(2.5)!r}"   # CPython: 2 (even)

print("surface OK")
