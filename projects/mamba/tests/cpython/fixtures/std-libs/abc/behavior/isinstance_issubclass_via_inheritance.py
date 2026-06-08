# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "isinstance_issubclass_via_inheritance"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: a real subclass of an ABC passes isinstance and issubclass against the ABC base"""
import abc


class Shape(abc.ABC):
    @abc.abstractmethod
    def area(self) -> float: ...


class Circle(Shape):
    def __init__(self, r: float):
        self.r = r
    def area(self) -> float:
        return 3.141592653589793 * self.r * self.r


c = Circle(1.0)
assert abs(c.area() - 3.141592653589793) < 1e-10, f"circle area: {c.area()!r}"
assert isinstance(c, Shape), "Circle instance isinstance Shape"
assert issubclass(Circle, Shape), "Circle issubclass Shape"

print("isinstance_issubclass_via_inheritance OK")
