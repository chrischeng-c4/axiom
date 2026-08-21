# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_invalid_nodes_in_done"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_invalid_nodes_in_done"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_invalid_nodes_in_done
"""Auto-ported test: TestTopologicalSort::test_invalid_nodes_in_done (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
ts.add(1, 2, 3, 4)
ts.add(2, 3, 4)
ts.prepare()
ts.get_ready()
try:
    ts.done(2)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('node 2 was not passed out', str(_aR_e))
try:
    ts.done(24)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('node 24 was not added using add\\(\\)', str(_aR_e))
print("TestTopologicalSort::test_invalid_nodes_in_done: ok")
