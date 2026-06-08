# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "simple_cycle_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: simple_cycle_raises (errors)."""
import graphlib

_raised = False
try:
    list(graphlib.TopologicalSorter({'A': {'B'}, 'B': {'A'}}).static_order())
except graphlib.CycleError:
    _raised = True
assert _raised, "simple_cycle_raises: expected graphlib.CycleError"
print("simple_cycle_raises OK")
