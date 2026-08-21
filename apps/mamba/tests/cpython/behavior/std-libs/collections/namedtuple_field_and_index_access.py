# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_field_and_index_access"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: a namedtuple instance exposes fields by name and by index, prints its canonical Name(field=value) repr, and supports both positional and keyword construction"""
from collections import namedtuple

Point = namedtuple("Point", ["x", "y"])
p = Point(1, 2)
assert p.x == 1 and p.y == 2, "field access"
assert p[0] == 1 and p[1] == 2, "index access"
assert repr(p) == "Point(x=1, y=2)", f"repr = {repr(p)!r}"
assert Point(x=3, y=4) == (3, 4), "keyword construction"
assert Point(5, y=6) == (5, 6), "mixed positional/keyword construction"

print("namedtuple_field_and_index_access OK")
