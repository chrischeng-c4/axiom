"""Behavior contract for builtins.isinstance.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: basic type checks
assert isinstance(42, int) is True
assert isinstance(3.14, float) is True
assert isinstance("hi", str) is True
assert isinstance(b"hi", bytes) is True
assert isinstance([], list) is True
assert isinstance((), tuple) is True
assert isinstance({}, dict) is True
assert isinstance(set(), set) is True
assert isinstance(frozenset(), frozenset) is True
assert isinstance(None, type(None)) is True

# Rule 2: bool is subtype of int
assert isinstance(True, bool) is True
assert isinstance(True, int) is True
assert isinstance(False, bool) is True
assert isinstance(False, int) is True

# Rule 3: int is NOT bool
assert isinstance(1, bool) is False
assert isinstance(0, bool) is False

# Rule 4: subclass relationship
class Animal: pass
class Dog(Animal): pass
d = Dog()
assert isinstance(d, Dog) is True
assert isinstance(d, Animal) is True
assert isinstance(d, object) is True

# Rule 5: tuple of types (OR logic)
assert isinstance(1, (int, str)) is True
assert isinstance("x", (int, str)) is True
assert isinstance(1.0, (int, str)) is False
assert isinstance(None, (int, str, type(None))) is True

# Rule 6: empty tuple returns False
assert isinstance(1, ()) is False

# Rule 7: TypeError for non-type second arg
_raised = False
try:
    isinstance(1, 42)  # type: ignore[arg-type]
except TypeError:
    _raised = True
assert _raised, "isinstance(1, 42) did not raise TypeError"

# Rule 8: isinstance with ABC-style __instancecheck__
class _Meta(type):
    def __instancecheck__(cls, instance):
        return isinstance(instance, int)
class _IntLike(metaclass=_Meta):
    pass
assert isinstance(42, _IntLike) is True, "custom __instancecheck__ failed"
assert isinstance("x", _IntLike) is False, "custom __instancecheck__ false-positive"

# Rule 9: nested tuple of types (CPython flattens)
assert isinstance(1, ((int,), str)) is True, "nested tuple isinstance"

print("behavior OK")
