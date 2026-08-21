# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "counter_non_iterable_init_raises"
# subject = "collections.Counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: counter_non_iterable_init_raises (errors)."""
import collections

_raised = False
try:
    collections.Counter(123)
except TypeError:
    _raised = True
assert _raised, "counter_non_iterable_init_raises: expected TypeError"
print("counter_non_iterable_init_raises OK")
