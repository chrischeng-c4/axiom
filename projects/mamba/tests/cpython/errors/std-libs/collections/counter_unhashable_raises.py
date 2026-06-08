# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "counter_unhashable_raises"
# subject = "collections.Counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: counter_unhashable_raises (errors)."""
import collections

_raised = False
try:
    hash(collections.Counter(a=1))
except TypeError:
    _raised = True
assert _raised, "counter_unhashable_raises: expected TypeError"
print("counter_unhashable_raises OK")
