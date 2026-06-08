# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heapreplace_pops_then_pushes"
# subject = "heapq.heapreplace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapreplace: heapreplace returns the current smallest and then inserts the new value; replacing the root of [1,2,3] with 0 returns 1 and leaves 0 at the root"""
import heapq

_h3 = [1, 2, 3]
heapq.heapify(_h3)
_old3 = heapq.heapreplace(_h3, 0)
assert _old3 == 1, f"heapreplace old = {_old3!r}"
# After replacing with 0, the heap min (root) is 0.
assert _h3[0] == 0, f"heapreplace new root = {_h3[0]!r}"
print("heapreplace_pops_then_pushes OK")
