# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heappushpop_single_element_larger"
# subject = "heapq.heappushpop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop on a one-element heap with a larger value returns the old min and the heap now holds only the new value"""
import heapq

_one = [10]
assert heapq.heappushpop(_one, 11) == 10, "pushpop larger returns old min"
assert _one == [11], f"heap now holds new value = {_one!r}"
print("heappushpop_single_element_larger OK")
