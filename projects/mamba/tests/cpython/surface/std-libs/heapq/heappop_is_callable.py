# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "heappop_is_callable"
# subject = "heapq.heappop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappop: heappop_is_callable (surface)."""
import heapq

assert callable(heapq.heappop)
print("heappop_is_callable OK")
