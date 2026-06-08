# Class system conformance: class instance attribute access (R4.2).
# Tests instance attribute set/get and method calls on objects.

class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def magnitude_sq(self):
        return self.x * self.x + self.y * self.y

    def describe(self):
        return f"Point({self.x}, {self.y})"

p = Point(3, 4)
print(p.x)
print(p.y)
print(p.magnitude_sq())
print(p.describe())

# Modify instance attributes
p.x = 10
print(p.x)
print(p.describe())

# Multiple instances
p2 = Point(1, 2)
print(p2.x)
print(p2.y)
print(p2.magnitude_sq())
