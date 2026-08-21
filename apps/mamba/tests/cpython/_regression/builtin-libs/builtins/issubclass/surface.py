"""Surface contract for builtins.issubclass.

# type-regime: monomorphic

Probes: name presence, callable, returns bool, single and tuple forms,
reflexive (T is subclass of T), TypeError on non-type first arg.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "issubclass"), "builtins.issubclass missing"
assert builtins.issubclass is issubclass, "builtins.issubclass is issubclass divergence"
assert callable(builtins.issubclass), "builtins.issubclass not callable"

# issubclass returns bool
assert isinstance(issubclass(int, object), bool), "issubclass returns bool"

# Reflexive: every type is a subclass of itself
assert issubclass(int, int) is True, "int issubclass of int"
assert issubclass(str, str) is True, "str issubclass of str"
assert issubclass(object, object) is True, "object issubclass of object"

# Known hierarchy
assert issubclass(bool, int) is True, "bool issubclass of int"
assert issubclass(int, object) is True, "int issubclass of object"
assert issubclass(str, object) is True, "str issubclass of object"

# Non-subclass
assert issubclass(str, int) is False, "str not issubclass of int"

# Tuple form
assert issubclass(int, (str, int)) is True, "int issubclass of (str,int)"
assert issubclass(bool, (int, str)) is True, "bool issubclass of (int,str)"
assert issubclass(list, (str, int)) is False, "list not issubclass of (str,int)"

# TypeError: first arg must be a type
_raised = False
try:
    issubclass(42, int)  # type: ignore[arg-type]
except TypeError:
    _raised = True
assert _raised, "issubclass(42, int) did not raise TypeError"

print("surface OK")
