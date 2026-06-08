"""Behavior contract for language __slots__.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: Slotted instances use less memory (no __dict__)
class _Slotted:
    __slots__ = ("a", "b", "c")
    def __init__(self, a: int, b: int, c: int):
        self.a = a
        self.b = b
        self.c = c

class _Normal:
    def __init__(self, a: int, b: int, c: int):
        self.a = a
        self.b = b
        self.c = c

_s = _Slotted(1, 2, 3)
_n = _Normal(1, 2, 3)
assert not hasattr(_s, "__dict__"), "slotted has no __dict__"
assert hasattr(_n, "__dict__"), "normal has __dict__"

# Rule 2: Accessing undefined slot raises AttributeError
_raised = False
try:
    _ = _s.d  # type: ignore[attr-defined]
except AttributeError:
    _raised = True
assert _raised, "undefined slot raises AttributeError"

# Rule 3: Setting undefined slot raises AttributeError
_raised2 = False
try:
    _s.d = 4  # type: ignore[attr-defined]
except AttributeError:
    _raised2 = True
assert _raised2, "setting undefined slot raises AttributeError"

# Rule 4: Slot descriptors are on the class
_xa = _Slotted.a
assert hasattr(_xa, "__get__"), "slot descriptor has __get__"
assert hasattr(_xa, "__set__"), "slot descriptor has __set__"

# Rule 5: Deleting a slot attribute raises AttributeError on next access
_s2 = _Slotted(10, 20, 30)
del _s2.a
_raised3 = False
try:
    _ = _s2.a
except AttributeError:
    _raised3 = True
assert _raised3, "deleted slot raises AttributeError on access"

# Rule 6: __slots__ = () prevents any instance attributes
class _NoAttrs:
    __slots__ = ()

_empty = _NoAttrs()
assert not hasattr(_empty, "__dict__"), "empty slots has no __dict__"
_raised4 = False
try:
    _empty.x = 1  # type: ignore[attr-defined]
except AttributeError:
    _raised4 = True
assert _raised4, "empty slots rejects any attr"

# Rule 7: Subclass with __slots__ inherits parent slots
class _Base:
    __slots__ = ("x",)
    def __init__(self, x: int):
        self.x = x

class _Child(_Base):
    __slots__ = ("y",)
    def __init__(self, x: int, y: int):
        super().__init__(x)
        self.y = y

_ch = _Child(1, 2)
assert _ch.x == 1, f"inherited slot x = {_ch.x!r}"
assert _ch.y == 2, f"child slot y = {_ch.y!r}"
assert not hasattr(_ch, "__dict__"), "fully slotted child has no __dict__"

# Rule 8: Slots work with classmethod and staticmethod
class _Counted:
    __slots__ = ("_val",)
    _class_count: int = 0  # class variable (not per-instance)

    def __init__(self, v: int):
        self._val = v
        _Counted._class_count += 1

    @classmethod
    def count(cls) -> int:
        return cls._class_count

_Counted._class_count = 0
_c1 = _Counted(1)
_c2 = _Counted(2)
assert _Counted.count() == 2, f"count = {_Counted.count()!r}"

print("behavior OK")
