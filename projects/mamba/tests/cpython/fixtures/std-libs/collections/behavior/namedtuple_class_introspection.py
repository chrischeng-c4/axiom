# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_class_introspection"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: the generated class exposes __name__, empty __slots__, _fields, __match_args__, and reuses tuple.__getitem__"""
from collections import namedtuple

Point = namedtuple("Point", "x y")
assert Point.__name__ == "Point", "name"
assert Point.__slots__ == (), "empty slots"
assert Point._fields == ("x", "y"), "fields tuple"
assert Point.__match_args__ == ("x", "y"), "match args"
assert Point.__getitem__ == tuple.__getitem__, "reuses tuple.__getitem__"

print("namedtuple_class_introspection OK")
