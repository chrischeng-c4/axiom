"""Surface contract for builtins.divmod.

# type-regime: monomorphic

Probes: name presence, callable, return type (tuple), int and float forms.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "divmod"), "builtins.divmod missing"
assert builtins.divmod is divmod, "builtins.divmod is divmod divergence"
assert callable(builtins.divmod), "builtins.divmod not callable"

# divmod returns a tuple
result = divmod(17, 5)
assert isinstance(result, tuple), f"divmod returns tuple: {type(result).__name__!r}"
assert len(result) == 2, f"divmod tuple len = {len(result)!r}"

# int divmod
q, r = divmod(17, 5)
assert isinstance(q, int), f"q is int: {type(q).__name__!r}"
assert isinstance(r, int), f"r is int: {type(r).__name__!r}"

# float divmod
q2, r2 = divmod(7.5, 2.5)
assert isinstance(q2, float), f"q2 is float: {type(q2).__name__!r}"
assert isinstance(r2, float), f"r2 is float: {type(r2).__name__!r}"

print("surface OK")
