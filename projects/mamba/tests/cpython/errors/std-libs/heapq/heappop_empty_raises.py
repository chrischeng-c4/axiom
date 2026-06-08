# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heappop_empty_raises"
# subject = "heapq.heappop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappop: heappop_empty_raises (errors)."""
import heapq

_raised = False
try:
    heapq.heappop([])
except IndexError:
    _raised = True
assert _raised, "heappop_empty_raises: expected IndexError"
print("heappop_empty_raises OK")
