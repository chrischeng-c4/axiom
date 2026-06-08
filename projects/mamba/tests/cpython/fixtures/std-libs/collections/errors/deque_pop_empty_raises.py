# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "deque_pop_empty_raises"
# subject = "collections.deque"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.deque: deque_pop_empty_raises (errors)."""
import collections

_raised = False
try:
    collections.deque().pop()
except IndexError:
    _raised = True
assert _raised, "deque_pop_empty_raises: expected IndexError"
print("deque_pop_empty_raises OK")
