# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "abstractmethods_set_contents"
# subject = "abc.ABC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: __abstractmethods__ lists exactly the unimplemented abstract names and is empty once all are overridden"""
import abc


class Shape(abc.ABC):
    @abc.abstractmethod
    def area(self) -> float: ...
    @abc.abstractmethod
    def perimeter(self) -> float: ...


# The base names both abstract methods.
assert "area" in Shape.__abstractmethods__, "area in __abstractmethods__"
assert "perimeter" in Shape.__abstractmethods__, "perimeter in __abstractmethods__"
assert set(Shape.__abstractmethods__) == {"area", "perimeter"}, "exact abstract set"


class Circle(Shape):
    def area(self) -> float:
        return 3.14
    def perimeter(self) -> float:
        return 6.28


# Overriding every abstract empties the set on the concrete subclass.
assert len(Circle.__abstractmethods__) == 0, "concrete subclass has empty __abstractmethods__"

print("abstractmethods_set_contents OK")
