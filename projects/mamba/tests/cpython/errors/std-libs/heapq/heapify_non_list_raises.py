# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heapify_non_list_raises"
# subject = "heapq.heapify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: heapify_non_list_raises (errors)."""
import heapq

_raised = False
try:
    heapq.heapify(10)
except TypeError:
    _raised = True
assert _raised, "heapify_non_list_raises: expected TypeError"
print("heapify_non_list_raises OK")
