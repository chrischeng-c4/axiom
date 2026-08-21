# abc.ABC / @abstractmethod — concrete subclasses override all abstracts
# and instantiate normally; diamond-like chain (Square → Rectangle → Shape).
from abc import ABC, abstractmethod

class Shape(ABC):
    @abstractmethod
    def area(self):
        pass

    @abstractmethod
    def perimeter(self):
        pass

class Rectangle(Shape):
    def __init__(self, w, h):
        self.w = w
        self.h = h

    def area(self):
        return self.w * self.h

    def perimeter(self):
        return 2 * (self.w + self.h)

class Square(Rectangle):
    def __init__(self, s):
        super().__init__(s, s)

r = Rectangle(3, 4)
print(r.area())
print(r.perimeter())

s = Square(5)
print(s.area())
print(s.perimeter())

# Polymorphic iteration over ABC subclasses
shapes = [Rectangle(2, 3), Square(4)]
for sh in shapes:
    print(sh.area())
