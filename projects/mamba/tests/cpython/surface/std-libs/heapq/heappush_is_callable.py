# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "heappush_is_callable"
# subject = "heapq.heappush"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heappush: heappush_is_callable (surface)."""
import heapq

assert callable(heapq.heappush)
print("heappush_is_callable OK")
