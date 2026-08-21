# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "done_unknown_node_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: done_unknown_node_raises (errors)."""
import graphlib

_raised = False
try:
    (lambda t: (t.add(1, 2), t.prepare(), t.done(24)))(graphlib.TopologicalSorter())
except ValueError:
    _raised = True
assert _raised, "done_unknown_node_raises: expected ValueError"
print("done_unknown_node_raises OK")
