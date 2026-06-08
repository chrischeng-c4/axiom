# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "tuples_ordered_lexicographically"
# subject = "heapq.heappush"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappush: tuples pushed onto a heap are compared lexicographically; popping yields them in ascending tuple order"""
import heapq

_th = []
heapq.heappush(_th, (2, "b"))
heapq.heappush(_th, (1, "a"))
heapq.heappush(_th, (3, "c"))
assert heapq.heappop(_th) == (1, "a"), "tuple heap smallest first"
assert heapq.heappop(_th) == (2, "b"), "tuple heap second"
print("tuples_ordered_lexicographically OK")
