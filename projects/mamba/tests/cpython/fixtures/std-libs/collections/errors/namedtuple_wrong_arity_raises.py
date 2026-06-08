# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "namedtuple_wrong_arity_raises"
# subject = "collections.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: namedtuple_wrong_arity_raises (errors)."""
import collections

_raised = False
try:
    collections.namedtuple('Point', 'x y')(1)
except TypeError:
    _raised = True
assert _raised, "namedtuple_wrong_arity_raises: expected TypeError"
print("namedtuple_wrong_arity_raises OK")
