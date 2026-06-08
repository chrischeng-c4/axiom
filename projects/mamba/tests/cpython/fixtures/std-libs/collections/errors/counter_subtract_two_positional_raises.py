# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "counter_subtract_two_positional_raises"
# subject = "collections.Counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: counter_subtract_two_positional_raises (errors)."""
import collections

_raised = False
try:
    collections.Counter().subtract({}, {})
except TypeError:
    _raised = True
assert _raised, "counter_subtract_two_positional_raises: expected TypeError"
print("counter_subtract_two_positional_raises OK")
