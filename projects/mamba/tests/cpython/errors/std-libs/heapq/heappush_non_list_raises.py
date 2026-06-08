# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heappush_non_list_raises"
# subject = "heapq.heappush"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappush: heappush_non_list_raises (errors)."""
import heapq

_raised = False
try:
    heapq.heappush(10, 10)
except TypeError:
    _raised = True
assert _raised, "heappush_non_list_raises: expected TypeError"
print("heappush_non_list_raises OK")
