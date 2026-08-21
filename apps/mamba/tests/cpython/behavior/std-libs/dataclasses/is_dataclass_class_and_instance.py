# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "is_dataclass_class_and_instance"
# subject = "dataclasses.is_dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.is_dataclass: is_dataclass is True for both the dataclass class and its instance, and False for a plain object()"""
import dataclasses


@dataclasses.dataclass
class Point:
    x: float
    y: float


p = Point(1.0, 2.0)
assert dataclasses.is_dataclass(p), "instance is_dataclass"
assert dataclasses.is_dataclass(Point), "class is_dataclass"
assert not dataclasses.is_dataclass(object()), "non-dataclass"

print("is_dataclass_class_and_instance OK")
