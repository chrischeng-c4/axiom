# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "nsmallest_is_callable"
# subject = "heapq.nsmallest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.nsmallest: nsmallest_is_callable (surface)."""
import heapq

assert callable(heapq.nsmallest)
print("nsmallest_is_callable OK")
