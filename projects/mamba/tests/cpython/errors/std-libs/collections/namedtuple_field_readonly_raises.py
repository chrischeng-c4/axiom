# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "namedtuple_field_readonly_raises"
# subject = "collections.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: namedtuple_field_readonly_raises (errors)."""
import collections

_raised = False
try:
    setattr(collections.namedtuple('Point', 'x y')(1, 2), 'x', 3)
except AttributeError:
    _raised = True
assert _raised, "namedtuple_field_readonly_raises: expected AttributeError"
print("namedtuple_field_readonly_raises OK")
