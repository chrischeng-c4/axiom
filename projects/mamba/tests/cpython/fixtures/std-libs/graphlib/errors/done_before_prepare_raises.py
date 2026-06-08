# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "done_before_prepare_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: done_before_prepare_raises (errors)."""
import graphlib

_raised = False
try:
    graphlib.TopologicalSorter().done(3)
except ValueError:
    _raised = True
assert _raised, "done_before_prepare_raises: expected ValueError"
print("done_before_prepare_raises OK")
