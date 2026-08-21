"""Behavior contract for language classes.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: __init__ sets instance attributes
class _Box:
    def __init__(self, w: int, h: int):
        self.w = w
        self.h = h
    def area(self) -> int:
        return self.w * self.h
b = _Box(3, 4)
assert b.w == 3 and b.h == 4
assert b.area() == 12

# Rule 2: instance attribute shadows class attribute
class _Shared:
    value = 100
s = _Shared()
assert s.value == 100      # sees class attr
s.value = 200              # shadows
assert s.value == 200
assert _Shared.value == 100  # class attr unchanged

# Rule 3: inheritance passes methods
class _Animal:
    def speak(self) -> str:
        return "..."
class _Dog(_Animal):
    def speak(self) -> str:
        return "woof"
class _Cat(_Animal):
    pass  # inherits speak

d = _Dog()
c = _Cat()
assert d.speak() == "woof", "override failed"
assert c.speak() == "...", "inherited failed"

# Rule 4: super() calls parent method
class _Shape:
    def describe(self) -> str:
        return "shape"
class _Circle(_Shape):
    def describe(self) -> str:
        return super().describe() + " circle"
assert _Circle().describe() == "shape circle"

# Rule 5: multiple inheritance — MRO
class A:
    def method(self) -> str:
        return "A"
class B(A):
    def method(self) -> str:
        return "B"
class C(A):
    def method(self) -> str:
        return "C"
class D(B, C):
    pass
d_obj = D()
assert d_obj.method() == "B", f"MRO = {D.__mro__}"
assert D.__mro__ == (D, B, C, A, object), f"MRO = {D.__mro__!r}"

# Rule 6: classmethod receives class, not instance
class _Factory:
    _count = 0
    @classmethod
    def make(cls) -> "_Factory":
        cls._count += 1
        return cls()
obj1 = _Factory.make()
obj2 = _Factory.make()
assert _Factory._count == 2

# Rule 7: staticmethod has no cls/self
class _Util:
    @staticmethod
    def square(x: int) -> int:
        return x * x
assert _Util.square(5) == 25
assert _Util().square(5) == 25

# Rule 8: __repr__ and __str__
class _Named:
    def __init__(self, name: str):
        self.name = name
    def __repr__(self) -> str:
        return f"Named({self.name!r})"
    def __str__(self) -> str:
        return self.name
n = _Named("Alice")
assert repr(n) == "Named('Alice')", f"repr = {repr(n)!r}"
assert str(n) == "Alice", f"str = {str(n)!r}"

# Rule 9: __eq__ and __hash__
class _Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, _Point):
            return NotImplemented
        return self.x == other.x and self.y == other.y
    def __hash__(self) -> int:
        return hash((self.x, self.y))
assert _Point(1, 2) == _Point(1, 2)
assert _Point(1, 2) != _Point(1, 3)
assert hash(_Point(1, 2)) == hash(_Point(1, 2))

# Rule 10: __len__ and __getitem__ make sequence-like
class _FixedList:
    def __init__(self, items: list):
        self._items = items
    def __len__(self) -> int:
        return len(self._items)
    def __getitem__(self, idx: int) -> int:
        return self._items[idx]
fl = _FixedList([10, 20, 30])
assert len(fl) == 3
assert fl[1] == 20
assert fl[-1] == 30

print("behavior OK")
