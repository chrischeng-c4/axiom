# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heappushpop_empty_no_raise"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop on an empty heap does NOT raise: with no root to compare, the pushed item is returned unchanged"""
import heapq

# Unlike heappop/heapreplace, heappushpop on an empty heap is well-defined:
# the pushed item is returned and the heap stays empty.
_h = []
_out = heapq.heappushpop(_h, 42)
assert _out == 42, f"heappushpop([], 42) = {_out!r}"
assert _h == [], f"heap stays empty = {_h!r}"
print("heappushpop_empty_no_raise OK")
