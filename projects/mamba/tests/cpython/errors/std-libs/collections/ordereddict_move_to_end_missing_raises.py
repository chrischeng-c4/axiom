# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "ordereddict_move_to_end_missing_raises"
# subject = "collections.OrderedDict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.OrderedDict: ordereddict_move_to_end_missing_raises (errors)."""
import collections

_raised = False
try:
    collections.OrderedDict([('a', 1)]).move_to_end('missing')
except KeyError:
    _raised = True
assert _raised, "ordereddict_move_to_end_missing_raises: expected KeyError"
print("ordereddict_move_to_end_missing_raises OK")
