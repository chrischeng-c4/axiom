# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heappushpop_smaller_returns_input"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop with a value smaller than the current min returns that value immediately and leaves the heap unchanged"""
import heapq

_h4 = [5, 6, 7]
heapq.heapify(_h4)
_r4 = heapq.heappushpop(_h4, 2)
assert _r4 == 2, f"heappushpop(2 < 5) = {_r4!r}"
assert _h4[0] == 5, f"heap unchanged min = {_h4[0]!r}"
print("heappushpop_smaller_returns_input OK")
