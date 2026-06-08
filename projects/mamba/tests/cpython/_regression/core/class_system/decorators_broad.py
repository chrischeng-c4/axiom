# classmethod/staticmethod/property broad

class Counter:
    count = 0
    def __init__(self):
        Counter.count += 1

    @classmethod
    def current(cls):
        return cls.count

print(Counter.current())
Counter()
Counter()
Counter()
print(Counter.current())

# classmethod called on instance
c = Counter()
print(c.current())

# @property
class Rect:
    def __init__(self, w, h):
        self.w = w
        self.h = h

    @property
    def area(self):
        return self.w * self.h

    @property
    def perimeter(self):
        return 2 * (self.w + self.h)

r = Rect(3, 4)
print(r.area)
print(r.perimeter)

r2 = Rect(5, 6)
print(r2.area)

# property accessed via method
class Circle:
    def __init__(self, radius):
        self.radius = radius

    @property
    def diameter(self):
        return self.radius * 2

c1 = Circle(5)
print(c1.diameter)
print(c1.radius)

# @staticmethod returning int
class MathBox:
    @staticmethod
    def square(n):
        return n * n

    @staticmethod
    def cube(n):
        return n * n * n

print(MathBox.square(5))
print(MathBox.cube(3))
print(MathBox.square(10))
