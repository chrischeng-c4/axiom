# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "is_active_before_prepare_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: is_active_before_prepare_raises (errors)."""
import graphlib

_raised = False
try:
    graphlib.TopologicalSorter().is_active()
except ValueError:
    _raised = True
assert _raised, "is_active_before_prepare_raises: expected ValueError"
print("is_active_before_prepare_raises OK")
