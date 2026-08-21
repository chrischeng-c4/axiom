"""Surface contract for builtins.callable.

# type-regime: monomorphic

Probes: name presence, callable, returns bool, works on functions/classes/
lambdas/instances-with-__call__/non-callables.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "callable"), "builtins.callable missing"
assert builtins.callable is callable, "builtins.callable is callable divergence"
assert callable(builtins.callable), "builtins.callable not callable"

# callable returns bool
assert isinstance(callable(len), bool), "callable(len) not bool"
assert isinstance(callable(42), bool), "callable(42) not bool"

# callable on callables → True
assert callable(len) is True, "callable(len) = True failed"
assert callable(print) is True, "callable(print) = True failed"
assert callable(int) is True, "callable(int) = True failed"
assert callable(lambda: None) is True, "callable(lambda) = True failed"

# callable on non-callables → False
assert callable(42) is False, "callable(42) = False failed"
assert callable("hello") is False, "callable('hello') = False failed"
assert callable(None) is False, "callable(None) = False failed"
assert callable([]) is False, "callable([]) = False failed"

print("surface OK")
