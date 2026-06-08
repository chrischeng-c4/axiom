# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "type-strict"
# lib = "param_types"
# dimension = "errors"
# case = "heapq_heappush_rejects_int_heap_argument"
# subject = "heapq.heappush"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""heapq.heappush: heapq_heappush_rejects_int_heap_argument (errors)."""
import heapq

try:
    result = heapq.heappush(1, 2)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
