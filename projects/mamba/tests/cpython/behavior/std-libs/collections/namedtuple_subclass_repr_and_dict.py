# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_subclass_repr_and_dict"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: a subclass of a namedtuple keeps namedtuple repr (using the subclass name) and may carry instance attributes in __dict__"""
from collections import namedtuple

class Named(namedtuple("_Named", ["a", "b"])):
    pass

assert repr(Named(1, 2)) == "Named(a=1, b=2)", "subclass repr uses the subclass name"
n = Named(3, 4)
n.extra = 5
assert n.__dict__ == {"extra": 5}, "subclass instances carry a __dict__"

print("namedtuple_subclass_repr_and_dict OK")
