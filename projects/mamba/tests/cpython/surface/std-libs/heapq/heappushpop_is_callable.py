# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "heappushpop_is_callable"
# subject = "heapq.heappushpop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappushpop: heappushpop_is_callable (surface)."""
import heapq

assert callable(heapq.heappushpop)
print("heappushpop_is_callable OK")
