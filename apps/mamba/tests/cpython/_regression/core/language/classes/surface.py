"""Surface contract for language classes.

# type-regime: monomorphic

Probes: class definition, __init__, instance/class attrs, methods,
inheritance, super(), isinstance, issubclass, __str__/__repr__,
class attrs vs instance attrs, staticmethod, classmethod.
CPython 3.12 is the oracle.
"""

# Basic class definition
class _Point:
    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y
    def __repr__(self) -> str:
        return f"_Point({self.x}, {self.y})"

p = _Point(3.0, 4.0)
assert isinstance(p, _Point), "isinstance failed"
assert p.x == 3.0 and p.y == 4.0, "init attrs"
assert "_Point(3.0, 4.0)" in repr(p), f"repr = {repr(p)!r}"

# Inheritance
class _Point3D(_Point):
    def __init__(self, x: float, y: float, z: float):
        super().__init__(x, y)
        self.z = z
p3 = _Point3D(1.0, 2.0, 3.0)
assert isinstance(p3, _Point3D)
assert isinstance(p3, _Point)
assert issubclass(_Point3D, _Point)

# Class attribute
class _Counter:
    count = 0
    def inc(self) -> None:
        _Counter.count += 1
c1 = _Counter()
c2 = _Counter()
c1.inc(); c1.inc()
assert _Counter.count == 2, f"class attr = {_Counter.count!r}"
assert c2.count == 2, "instance sees class attr"

# staticmethod
class _Math:
    @staticmethod
    def add(a: int, b: int) -> int:
        return a + b
assert _Math.add(2, 3) == 5
assert _Math().add(2, 3) == 5

# classmethod
class _Base:
    @classmethod
    def create(cls) -> "_Base":
        return cls()
obj = _Base.create()
assert isinstance(obj, _Base)

print("surface OK")
