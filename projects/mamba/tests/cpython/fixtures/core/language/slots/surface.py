"""Surface contract for language __slots__.

# type-regime: monomorphic

Probes: __slots__ declaration, attribute access, no __dict__,
AttributeError on unknown attrs, inheritance with slots.
CPython 3.12 is the oracle.
"""

# Basic __slots__
class _Point:
    __slots__ = ("x", "y")
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

_p = _Point(3, 4)
assert _p.x == 3, f"x = {_p.x!r}"
assert _p.y == 4, f"y = {_p.y!r}"

# __dict__ is absent on slotted instances
assert not hasattr(_p, "__dict__"), "__dict__ should not exist on slotted instance"

# __slots__ as class attribute
assert hasattr(_Point, "__slots__"), "__slots__ not on class"

# Setting undefined attr raises AttributeError
_raised = False
try:
    _p.z = 99  # type: ignore[attr-defined]
except AttributeError:
    _raised = True
assert _raised, "undeclared attr should raise AttributeError"

# Slots are descriptors on the class
assert hasattr(_Point, "x"), "x descriptor on class"
assert hasattr(_Point, "y"), "y descriptor on class"

# Reassignment works
_p.x = 10
assert _p.x == 10, f"reassign x = {_p.x!r}"

# Subclass of slotted class
class _Point3D(_Point):
    __slots__ = ("z",)
    def __init__(self, x: int, y: int, z: int):
        super().__init__(x, y)
        self.z = z

_p3 = _Point3D(1, 2, 3)
assert _p3.x == 1, f"p3.x = {_p3.x!r}"
assert _p3.y == 2, f"p3.y = {_p3.y!r}"
assert _p3.z == 3, f"p3.z = {_p3.z!r}"

# Subclass with no __slots__ gets __dict__
class _Flexible(_Point):
    pass  # no __slots__ → gets __dict__

_f = _Flexible(5, 6)
assert hasattr(_f, "__dict__"), "subclass without __slots__ gets __dict__"
_f.extra = "bonus"  # should work
assert _f.extra == "bonus", f"extra = {_f.extra!r}"

print("surface OK")
