# T2.1: @property getter invoked on attribute read
# Conformance test: must produce identical output under CPython 3.12 and Mamba.

class Circle:
    def __init__(self, r):
        self._r = r

    @property
    def area(self):
        return 3.14159 * self._r * self._r

    @property
    def radius(self):
        return self._r

c = Circle(5)
print(c.area)     # Expected: 78.53975
print(c.radius)   # Expected: 5

c2 = Circle(10)
print(c2.area)    # Expected: 314.159
