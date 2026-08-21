# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "nlargest_is_callable"
# subject = "heapq.nlargest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.nlargest: nlargest_is_callable (surface)."""
import heapq

assert callable(heapq.nlargest)
print("nlargest_is_callable OK")
