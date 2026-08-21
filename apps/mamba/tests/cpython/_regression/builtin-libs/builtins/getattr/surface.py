"""Surface contract for builtins.getattr / setattr / hasattr / delattr.

# type-regime: monomorphic

Probes: name presence, callable, return types, basic operation.
CPython 3.12 is the oracle.
"""

import builtins

# getattr
assert hasattr(builtins, "getattr"), "builtins.getattr missing"
assert builtins.getattr is getattr, "builtins.getattr is getattr divergence"
assert callable(builtins.getattr), "builtins.getattr not callable"

# setattr
assert hasattr(builtins, "setattr"), "builtins.setattr missing"
assert builtins.setattr is setattr, "builtins.setattr is setattr divergence"
assert callable(builtins.setattr), "builtins.setattr not callable"

# hasattr
assert hasattr(builtins, "hasattr"), "builtins.hasattr missing"
assert builtins.hasattr is hasattr, "builtins.hasattr is hasattr divergence"
assert callable(builtins.hasattr), "builtins.hasattr not callable"

# delattr
assert hasattr(builtins, "delattr"), "builtins.delattr missing"
assert builtins.delattr is delattr, "builtins.delattr is delattr divergence"
assert callable(builtins.delattr), "builtins.delattr not callable"

# hasattr returns bool
class _C:
    x = 1
assert isinstance(hasattr(_C, "x"), bool), "hasattr returns bool"
assert isinstance(hasattr(_C, "z"), bool), "hasattr returns bool (missing)"

# getattr with default
assert getattr(_C, "x") == 1, "getattr existing"
assert getattr(_C, "z", 99) == 99, "getattr default"

print("surface OK")
