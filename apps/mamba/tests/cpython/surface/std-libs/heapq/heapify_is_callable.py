# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "heapify_is_callable"
# subject = "heapq.heapify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: heapify_is_callable (surface)."""
import heapq

assert callable(heapq.heapify)
print("heapify_is_callable OK")
