"""Behavior contract for builtins.issubclass.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: reflexive — every type is its own subclass
assert issubclass(int, int) is True
assert issubclass(str, str) is True
assert issubclass(object, object) is True
assert issubclass(list, list) is True
assert issubclass(bool, bool) is True

# Rule 2: known hierarchy
assert issubclass(bool, int) is True
assert issubclass(int, object) is True
assert issubclass(float, object) is True
assert issubclass(str, object) is True
assert issubclass(list, object) is True
assert issubclass(dict, object) is True

# Rule 3: non-subclass relationships
assert issubclass(str, int) is False
assert issubclass(int, str) is False
assert issubclass(list, dict) is False
assert issubclass(float, int) is False

# Rule 4: user-defined class hierarchy
class A: pass
class B(A): pass
class C(B): pass
class D: pass
assert issubclass(B, A) is True
assert issubclass(C, A) is True   # transitivity
assert issubclass(C, B) is True
assert issubclass(A, B) is False  # not reversed
assert issubclass(D, A) is False  # unrelated

# Rule 5: tuple form — OR semantics
assert issubclass(int, (str, int)) is True
assert issubclass(bool, (int, str)) is True
assert issubclass(list, (int, str)) is False
assert issubclass(list, (list, dict)) is True

# Rule 6: empty tuple always False
assert issubclass(int, ()) is False

# Rule 7: TypeError for non-type first arg
for bad in (42, "x", None, 3.14):
    _raised = False
    try:
        issubclass(bad, object)  # type: ignore[arg-type]
    except TypeError:
        _raised = True
    assert _raised, f"issubclass({bad!r}, object) did not raise TypeError"

# Rule 8: TypeError for non-type second arg
_raised = False
try:
    issubclass(int, 42)  # type: ignore[arg-type]
except TypeError:
    _raised = True
assert _raised, "issubclass(int, 42) did not raise TypeError"

# Rule 9: multiple inheritance — diamond
class X: pass
class Y(X): pass
class Z(X): pass
class W(Y, Z): pass
assert issubclass(W, X) is True
assert issubclass(W, Y) is True
assert issubclass(W, Z) is True

# Rule 10: custom __subclasscheck__
class _Meta(type):
    def __subclasscheck__(cls, sub: type) -> bool:
        return True  # everything is a subclass
class _Base(metaclass=_Meta):
    pass
assert issubclass(int, _Base) is True, "custom __subclasscheck__ failed"

print("behavior OK")
