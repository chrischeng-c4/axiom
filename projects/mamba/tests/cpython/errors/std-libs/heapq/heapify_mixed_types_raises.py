# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heapify_mixed_types_raises"
# subject = "heapq.heapify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: heapify_mixed_types_raises (errors)."""
import heapq

_raised = False
try:
    heapq.heapify([1, "two", 3])
except TypeError:
    _raised = True
assert _raised, "heapify_mixed_types_raises: expected TypeError"
print("heapify_mixed_types_raises OK")
