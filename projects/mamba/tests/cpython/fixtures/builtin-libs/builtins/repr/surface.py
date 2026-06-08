"""Surface contract for builtins.repr.

# type-regime: monomorphic

Probes: name presence, callable, returns str, works on basic types.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "repr"), "builtins.repr missing"
assert builtins.repr is repr, "builtins.repr is repr divergence"
assert callable(builtins.repr), "builtins.repr not callable"

# repr returns str for all basic types
assert isinstance(repr(42), str), "repr(42) not str"
assert isinstance(repr(3.14), str), "repr(3.14) not str"
assert isinstance(repr("hello"), str), "repr('hello') not str"
assert isinstance(repr([1, 2]), str), "repr([1,2]) not str"
assert isinstance(repr(None), str), "repr(None) not str"
assert isinstance(repr(True), str), "repr(True) not str"

# Known repr values from CPython 3.12
assert repr(42) == "42", f"repr(42) = {repr(42)!r}"
assert repr(None) == "None", f"repr(None) = {repr(None)!r}"
assert repr(True) == "True", f"repr(True) = {repr(True)!r}"
assert repr("hi") == "'hi'", f"repr('hi') = {repr('hi')!r}"

print("surface OK")
