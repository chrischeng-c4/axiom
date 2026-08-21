"""Behavior contract for builtins.getattr / setattr / hasattr / delattr.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

class _Obj:
    x = 10

# Rule 1: getattr — existing attribute
assert getattr(_Obj, "x") == 10, f"getattr existing = {getattr(_Obj, 'x')!r}"

# Rule 2: getattr — missing raises AttributeError
_raised = False
try:
    getattr(_Obj, "z")
except AttributeError:
    _raised = True
assert _raised, "getattr missing did not raise AttributeError"

# Rule 3: getattr — missing with default returns default
assert getattr(_Obj, "z", 99) == 99, f"getattr default = {getattr(_Obj, 'z', 99)!r}"
assert getattr(_Obj, "z", None) is None, "getattr default None"

# Rule 4: setattr — sets attribute on instance
obj = _Obj()
setattr(obj, "y", 42)
assert obj.y == 42, f"setattr y = {obj.y!r}"  # type: ignore[attr-defined]

# Rule 5: setattr — overwrites existing
setattr(obj, "x", 99)
assert obj.x == 99, f"setattr overwrite = {obj.x!r}"

# Rule 6: hasattr — existing → True
assert hasattr(_Obj, "x") is True, "hasattr existing"
assert hasattr(_Obj, "__class__") is True, "hasattr __class__"

# Rule 7: hasattr — missing → False
assert hasattr(_Obj, "z") is False, "hasattr missing"

# Rule 8: hasattr — catches AttributeError from __getattr__
class _Raises:
    def __getattr__(self, name: str):
        raise AttributeError(name)
assert hasattr(_Raises(), "anything") is False, "hasattr swallows AttributeError"

# Rule 9: delattr — removes instance attribute
obj2 = _Obj()
obj2.extra = "hi"  # type: ignore[attr-defined]
assert hasattr(obj2, "extra") is True, "extra not set"
delattr(obj2, "extra")
assert hasattr(obj2, "extra") is False, "delattr failed"

# Rule 10: delattr — missing raises AttributeError
_raised = False
try:
    delattr(_Obj(), "nonexistent")
except AttributeError:
    _raised = True
assert _raised, "delattr missing did not raise AttributeError"

# Rule 11: getattr on built-in type
assert getattr(int, "__name__") == "int", "getattr int.__name__"
assert getattr(str, "upper") is str.upper, "getattr str.upper"

# Rule 12: getattr with dunder
assert getattr([], "__class__") is list, "getattr [].__class__"

print("behavior OK")
