# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "namedtuple_fields_and_tuple_shape"
# subject = "typing.NamedTuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.NamedTuple: a class-syntax NamedTuple (Point: x, y) builds an instance addressable by field name and by index, equal to the plain tuple of its values"""
import typing


class Point(typing.NamedTuple):
    x: int
    y: int


p = Point(1, 2)
assert p.x == 1 and p.y == 2, "fields addressable by name"
assert p[0] == 1 and p[1] == 2, "fields addressable by index"
assert tuple(p) == (1, 2), "a NamedTuple is the plain tuple of its values"
assert p == (1, 2), "a NamedTuple compares equal to the plain tuple"
print("namedtuple_fields_and_tuple_shape OK")
