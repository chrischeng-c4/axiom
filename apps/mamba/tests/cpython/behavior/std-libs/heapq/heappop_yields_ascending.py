# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heappop_yields_ascending"
# subject = "heapq.heappop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappop: draining a heapified list via repeated heappop yields the elements in ascending (sorted) order"""
import heapq

_h = [5, 3, 7, 1, 9, 2]
heapq.heapify(_h)
_popped = []
while _h:
    _popped.append(heapq.heappop(_h))
assert _popped == sorted([5, 3, 7, 1, 9, 2]), f"heap sort order = {_popped!r}"
print("heappop_yields_ascending OK")
