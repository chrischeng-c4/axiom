# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "errors"
# case = "heapreplace_empty_raises"
# subject = "heapq.heapreplace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapreplace: heapreplace_empty_raises (errors)."""
import heapq

_raised = False
try:
    heapq.heapreplace([], 1)
except IndexError:
    _raised = True
assert _raised, "heapreplace_empty_raises: expected IndexError"
print("heapreplace_empty_raises OK")
