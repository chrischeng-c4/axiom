# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "errors"
# case = "done_not_passed_out_raises"
# subject = "graphlib.TopologicalSorter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: done_not_passed_out_raises (errors)."""
import graphlib

_raised = False
try:
    (lambda t: (t.add(1, 2, 3, 4), t.add(2, 3, 4), t.prepare(), t.get_ready(), t.done(2)))(graphlib.TopologicalSorter())
except ValueError:
    _raised = True
assert _raised, "done_not_passed_out_raises: expected ValueError"
print("done_not_passed_out_raises OK")
