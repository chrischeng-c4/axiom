# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "counter_fromkeys_raises"
# subject = "collections.Counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: counter_fromkeys_raises (errors)."""
import collections

_raised = False
try:
    collections.Counter.fromkeys('abc')
except NotImplementedError:
    _raised = True
assert _raised, "counter_fromkeys_raises: expected NotImplementedError"
print("counter_fromkeys_raises OK")
