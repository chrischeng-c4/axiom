# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "nlargest_non_iterable_raises"
# subject = "heapq.nlargest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.nlargest: nlargest_non_iterable_raises (errors)."""
import heapq

_raised = False
try:
    heapq.nlargest(3, 123)
except TypeError:
    _raised = True
assert _raised, "nlargest_non_iterable_raises: expected TypeError"
print("nlargest_non_iterable_raises OK")
