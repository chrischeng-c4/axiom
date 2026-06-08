# Class system conformance: single inheritance (R4.1).
# Tests single inheritance, method override, isinstance, super().__init__.

class Shape:
    def __init__(self, color):
        self.color = color

    def describe(self):
        return f"A {self.color} shape"

class Circle(Shape):
    def __init__(self, color, radius):
        super().__init__(color)
        self.radius = radius

    def describe(self):
        return f"A {self.color} circle"

c = Circle("red", 5)
print(c.describe())
print(c.color)
print(c.radius)
print(isinstance(c, Circle))
print(isinstance(c, Shape))

# Method inherited from parent
class Rectangle(Shape):
    def __init__(self, color, width, height):
        super().__init__(color)
        self.width = width
        self.height = height

    def area(self):
        return self.width * self.height

r = Rectangle("blue", 3, 4)
print(r.describe())
print(r.area())
print(r.color)
