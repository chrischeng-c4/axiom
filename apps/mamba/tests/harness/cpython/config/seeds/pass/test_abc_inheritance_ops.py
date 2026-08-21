# Operational AssertionPass seed for `abc.ABC` + `@abstractmethod`
# inheritance and method-dispatch surface.
# Surface: a concrete subclass that provides the abstract method is
# instantiable; instances dispatch the override; isinstance against
# the abstract base recognises the concrete subclass.
from abc import ABC, abstractmethod

class Shape(ABC):
    @abstractmethod
    def area(self):
        pass

class Square(Shape):
    def __init__(self, side):
        self.side = side
    def area(self):
        return self.side * self.side

class Rect(Shape):
    def __init__(self, w, h):
        self.w = w
        self.h = h
    def area(self):
        return self.w * self.h

_ledger: list[int] = []
s = Square(5)
r = Rect(3, 4)
# Concrete subclasses can be instantiated
assert s is not None; _ledger.append(1)
assert r is not None; _ledger.append(1)
# Override dispatches correctly per subclass
assert s.area() == 25; _ledger.append(1)
assert r.area() == 12; _ledger.append(1)
# isinstance recognises the abstract base as a parent
assert isinstance(s, Shape); _ledger.append(1)
assert isinstance(r, Shape); _ledger.append(1)
# Concrete-subclass identity preserved
assert isinstance(s, Square); _ledger.append(1)
assert isinstance(r, Rect); _ledger.append(1)
# Per-instance state preserved
assert s.side == 5; _ledger.append(1)
assert r.w == 3; _ledger.append(1)
assert r.h == 4; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_abc_inheritance_ops {sum(_ledger)} asserts")
