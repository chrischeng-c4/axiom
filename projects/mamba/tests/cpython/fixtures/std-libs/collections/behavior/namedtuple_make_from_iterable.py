# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_make_from_iterable"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: _make builds an instance from an iterable and enforces the field count, raising TypeError on the wrong length"""
from collections import namedtuple

Point = namedtuple("Point", "x y")
assert Point._make([11, 22]) == (11, 22), "_make from a list"
for bad in ([11], [11, 22, 33]):
    try:
        Point._make(bad)
        raise AssertionError(f"expected TypeError for _make({bad!r})")
    except TypeError:
        pass

print("namedtuple_make_from_iterable OK")
