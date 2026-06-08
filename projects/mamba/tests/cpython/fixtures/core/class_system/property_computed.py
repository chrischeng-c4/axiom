# Computed property: area from width * height

class Rectangle:
    def __init__(self, width, height):
        self._width = width
        self._height = height

    @property
    def width(self):
        return self._width

    @property
    def height(self):
        return self._height

    @property
    def area(self):
        return self._width * self._height

    @property
    def perimeter(self):
        return 2 * (self._width + self._height)

r = Rectangle(3, 4)
print(r.width)
print(r.height)
print(r.area)
print(r.perimeter)

r2 = Rectangle(10, 5)
print(r2.area)
print(r2.perimeter)
