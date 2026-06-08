# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "namedtuple_bad_field_name_raises"
# subject = "collections.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: namedtuple_bad_field_name_raises (errors)."""
import collections

_raised = False
try:
    collections.namedtuple('Bad', 'x class')
except ValueError:
    _raised = True
assert _raised, "namedtuple_bad_field_name_raises: expected ValueError"
print("namedtuple_bad_field_name_raises OK")
