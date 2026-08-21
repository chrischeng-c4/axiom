# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_not_hashable_nodes"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_not_hashable_nodes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_not_hashable_nodes
"""Auto-ported test: TestTopologicalSort::test_not_hashable_nodes (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()

try:
    ts.add(dict(), 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    ts.add(1, dict())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    ts.add(dict(), dict())
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestTopologicalSort::test_not_hashable_nodes: ok")
