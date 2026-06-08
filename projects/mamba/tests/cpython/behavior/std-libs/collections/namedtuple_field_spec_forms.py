# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_field_spec_forms"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: field names accept a space string, a comma string, or a sequence of names, all yielding the same _fields tuple"""
from collections import namedtuple

assert namedtuple("A", "p q")._fields == ("p", "q"), "space-separated string"
assert namedtuple("B", "p, q")._fields == ("p", "q"), "comma-separated string"
assert namedtuple("C", ("p", "q"))._fields == ("p", "q"), "tuple of names"
assert namedtuple("D", ["p", "q"])._fields == ("p", "q"), "list of names"

print("namedtuple_field_spec_forms OK")
