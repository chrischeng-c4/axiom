# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_is_a_real_tuple"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: a namedtuple instance is a genuine tuple: isinstance(tuple), unpacking, indexing incl. negative, max(), and hash() equal to the equivalent plain tuple"""
from collections import namedtuple

Point = namedtuple("Point", "x y")
p = Point(11, 22)
assert isinstance(p, tuple), "is a tuple"
assert tuple(p) == (11, 22) and list(p) == [11, 22], "convert"
assert p[0] == 11 and p[-1] == 22, "index incl. negative"
assert max(p) == 22, "max over elements"
x, y = p
assert (x, y) == (11, 22), "unpack"
assert hash(p) == hash((11, 22)), "hash matches the plain tuple"

print("namedtuple_is_a_real_tuple OK")
