# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "heapreplace_is_callable"
# subject = "heapq.heapreplace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapreplace: heapreplace_is_callable (surface)."""
import heapq

assert callable(heapq.heapreplace)
print("heapreplace_is_callable OK")
