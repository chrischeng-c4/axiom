# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_replace_and_asdict"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: _replace returns a modified copy leaving the original unchanged, and _asdict returns a plain dict of the fields"""
from collections import namedtuple

Pt = namedtuple("Pt", ["x", "y"])
p = Pt(1, 2)
p2 = p._replace(y=99)
assert p2.x == 1 and p2.y == 99, f"_replace = {p2!r}"
assert p.y == 2, "original unchanged by _replace"
d = p._asdict()
assert isinstance(d, dict) and d == {"x": 1, "y": 2}, f"_asdict = {d!r}"

print("namedtuple_replace_and_asdict OK")
