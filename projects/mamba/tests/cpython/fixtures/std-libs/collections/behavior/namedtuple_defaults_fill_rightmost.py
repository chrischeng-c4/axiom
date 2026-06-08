# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_defaults_fill_rightmost"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: the defaults= keyword fills the rightmost fields; _field_defaults reflects them; a single default leaves earlier fields required; defaults may be any iterable; defaults=None means none"""
from collections import namedtuple

Point = namedtuple("Point", "x y", defaults=(10, 20))
assert Point._field_defaults == {"x": 10, "y": 20}, "both fields defaulted"
assert Point(1, 2) == (1, 2) and Point(1) == (1, 20) and Point() == (10, 20), "defaults fill rightmost"

Partial = namedtuple("Partial", "x y", defaults=(20,))
assert Partial._field_defaults == {"y": 20}, "single default fills only the last field"
assert Partial(1) == (1, 20), "x still required"

FromIter = namedtuple("FromIter", "x y", defaults=iter([10, 20]))
assert FromIter() == (10, 20), "defaults may be any iterable"

NoneDef = namedtuple("NoneDef", "x y", defaults=None)
assert NoneDef._field_defaults == {}, "defaults=None means no defaults"

print("namedtuple_defaults_fill_rightmost OK")
