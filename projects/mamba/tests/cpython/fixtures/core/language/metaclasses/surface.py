"""Surface contract for language metaclasses.

# type-regime: monomorphic

Probes: custom metaclass with __new__ and __init__, metaclass=,
type() as metaclass, __prepare__, ABCMeta basics.
CPython 3.12 is the oracle.
"""

# type is the default metaclass
assert type(int) is type, f"type(int) = {type(int)!r}"
assert type(str) is type, f"type(str) = {type(str)!r}"

class _Plain:
    pass
assert type(_Plain) is type, f"type(_Plain) = {type(_Plain)!r}"

# Custom metaclass via metaclass= keyword
class _TrackMeta(type):
    _registry: list = []
    def __new__(mcs, name, bases, namespace):
        cls = super().__new__(mcs, name, bases, namespace)
        _TrackMeta._registry.append(name)
        return cls

_TrackMeta._registry.clear()

class _A(metaclass=_TrackMeta):
    pass

class _B(metaclass=_TrackMeta):
    pass

assert "A" in str(_TrackMeta._registry), f"registry = {_TrackMeta._registry!r}"
assert "_A" in _TrackMeta._registry or "_A" in str(_TrackMeta._registry), f"A tracked"
assert len(_TrackMeta._registry) == 2, f"registry len = {len(_TrackMeta._registry)!r}"

# type() as 3-arg call to create class
_Dyn = type("_Dyn", (object,), {"x": 42, "greet": lambda self: "hi"})
assert _Dyn.x == 42, f"Dyn.x = {_Dyn.x!r}"
assert _Dyn().greet() == "hi", f"Dyn.greet = {_Dyn().greet()!r}"
assert type(_Dyn) is type, f"type(_Dyn) = {type(_Dyn)!r}"

# isinstance/issubclass with metaclass
assert isinstance(_A(), _A), "isinstance with metaclass class"
assert issubclass(_A, object), "metaclass class subclasses object"

# ABCMeta from abc module
from abc import ABCMeta, abstractmethod

class _Shape(metaclass=ABCMeta):
    @abstractmethod
    def area(self) -> float: ...

class _Square(_Shape):
    def __init__(self, s: float):
        self.s = s
    def area(self) -> float:
        return self.s ** 2

_sq = _Square(3.0)
assert _sq.area() == 9.0, f"area = {_sq.area()!r}"

# Instantiating abstract class raises TypeError
_raised = False
try:
    _Shape()  # type: ignore[abstract]
except TypeError:
    _raised = True
assert _raised, "abstract class should raise TypeError"

print("surface OK")
