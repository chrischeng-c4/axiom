"""Surface contract for language descriptors.

# type-regime: monomorphic

Probes: data descriptor (__get__/__set__/__delete__), non-data descriptor
(__get__ only), property as built-in descriptor, classmethod, staticmethod.
CPython 3.12 is the oracle.
"""

# Non-data descriptor — __get__ only
class _LazyAttr:
    def __init__(self, func):
        self._func = func
        self._name = func.__name__
    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        value = self._func(obj)
        setattr(obj, self._name, value)  # cache on instance
        return value

class _Circle:
    def __init__(self, r: float):
        self.r = r

    @_LazyAttr
    def area(self) -> float:
        import math
        return math.pi * self.r ** 2

_c = _Circle(3.0)
assert hasattr(_Circle, "area"), "area descriptor on class"
_a = _c.area
assert abs(_a - 28.27433388230814) < 1e-5, f"area = {_a!r}"

# Data descriptor — __get__ and __set__
class _Validated:
    def __set_name__(self, owner, name: str):
        self._name = name
    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        return obj.__dict__.get(self._name, 0)
    def __set__(self, obj, value: int):
        if not isinstance(value, int):
            raise TypeError(f"{self._name} must be int")
        obj.__dict__[self._name] = value

class _Box:
    side = _Validated()

_b = _Box()
_b.side = 5
assert _b.side == 5, f"validated attr = {_b.side!r}"

_raised = False
try:
    _b.side = "oops"  # type: ignore[assignment]
except TypeError:
    _raised = True
assert _raised, "validator should reject non-int"

# property built-in descriptor
class _Temp:
    def __init__(self, c: float):
        self._c = c
    @property
    def celsius(self) -> float:
        return self._c
    @celsius.setter
    def celsius(self, v: float) -> None:
        self._c = v
    @property
    def fahrenheit(self) -> float:
        return self._c * 9/5 + 32

_t = _Temp(0.0)
assert _t.celsius == 0.0, f"celsius = {_t.celsius!r}"
assert _t.fahrenheit == 32.0, f"fahrenheit = {_t.fahrenheit!r}"
_t.celsius = 100.0
assert _t.fahrenheit == 212.0, f"100C in F = {_t.fahrenheit!r}"

# classmethod descriptor
class _Counter:
    _n = 0
    @classmethod
    def inc(cls) -> int:
        cls._n += 1
        return cls._n
    @classmethod
    def reset(cls) -> None:
        cls._n = 0

_Counter.reset()
assert _Counter.inc() == 1, "inc 1"
assert _Counter.inc() == 2, "inc 2"

# staticmethod descriptor
class _Util:
    @staticmethod
    def add(a: int, b: int) -> int:
        return a + b

assert _Util.add(3, 4) == 7, f"static add = {_Util.add(3,4)!r}"
assert _Util().add(3, 4) == 7, f"static add via instance = {_Util().add(3,4)!r}"

print("surface OK")
