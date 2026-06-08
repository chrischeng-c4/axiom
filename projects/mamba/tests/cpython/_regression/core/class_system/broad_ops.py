class Shape:
    def __init__(self, name):
        self.name = name
    def area(self):
        return 0
    def describe(self):
        return f"{self.name}: area={self.area()}"

class Circle(Shape):
    def __init__(self, radius):
        super().__init__("circle")
        self.radius = radius
    def area(self):
        return 3.14 * self.radius * self.radius

class Square(Shape):
    def __init__(self, side):
        super().__init__("square")
        self.side = side
    def area(self):
        return self.side * self.side

shapes = [Circle(5), Square(4), Circle(3)]
for s in shapes:
    print(s.describe())

print(isinstance(Circle(1), Shape))
print(isinstance(Circle(1), Circle))
print(isinstance(Square(1), Circle))

print(issubclass(Circle, Shape))
print(issubclass(Shape, Circle))
print(issubclass(Square, Shape))

class A:
    def greet(self):
        return "A"

class B(A):
    def greet(self):
        return "B-" + super().greet()

class C(B):
    def greet(self):
        return "C-" + super().greet()

print(A().greet())
print(B().greet())
print(C().greet())

class Counter:
    def __init__(self):
        self.count = 0
    def inc(self):
        self.count += 1
    def inc_by(self, n):
        self.count += n

c = Counter()
c.inc()
c.inc()
c.inc_by(10)
print(c.count)

# class variable vs instance variable
class Dog:
    species = "Canis familiaris"
    def __init__(self, name):
        self.name = name

d1 = Dog("Rex")
d2 = Dog("Buddy")
print(d1.species)
print(d2.species)
print(d1.name)
print(d2.name)

# explicit __str__ dispatch
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __str__(self):
        return f"({self.x}, {self.y})"

p = Point(3, 4)
print(p)
print(str(p))
