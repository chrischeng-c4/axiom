# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "unhashable_predecessor_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: unhashable_predecessor_raises (errors)."""
import graphlib

_raised = False
try:
    graphlib.TopologicalSorter().add({}, 1)
except TypeError:
    _raised = True
assert _raised, "unhashable_predecessor_raises: expected TypeError"
print("unhashable_predecessor_raises OK")
