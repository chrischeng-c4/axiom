# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heappushpop_larger_pops_min"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop with a value larger than the current min pops and returns the old min, and the new value is inserted into the heap"""
import heapq

_h5 = [1, 3, 5]
heapq.heapify(_h5)
_r5 = heapq.heappushpop(_h5, 10)
assert _r5 == 1, f"heappushpop(10 > 1) pops 1 = {_r5!r}"
assert 10 in _h5, "10 was inserted"
print("heappushpop_larger_pops_min OK")
