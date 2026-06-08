# @property getter

class Circle:
    def __init__(self, radius):
        self._radius = radius

    @property
    def radius(self):
        return self._radius

    @property
    def diameter(self):
        return self._radius * 2

    @property
    def area(self):
        return 3.14159 * self._radius * self._radius

c = Circle(5)
print(c.radius)
print(c.diameter)
print(c.area)

c2 = Circle(1)
print(c2.radius)
print(c2.diameter)
