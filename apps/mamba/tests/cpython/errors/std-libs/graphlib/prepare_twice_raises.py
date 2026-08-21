# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "prepare_twice_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: prepare_twice_raises (errors)."""
import graphlib

_raised = False
try:
    (lambda t: (t.prepare(), t.prepare()))(graphlib.TopologicalSorter())
except ValueError:
    _raised = True
assert _raised, "prepare_twice_raises: expected ValueError"
print("prepare_twice_raises OK")
