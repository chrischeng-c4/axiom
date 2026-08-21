# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_graph_with_iterables"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_graph_with_iterables"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_graph_with_iterables
"""Auto-ported test: TestTopologicalSort::test_graph_with_iterables (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
dependson = (2 * x + 1 for x in range(5))
ts = graphlib.TopologicalSorter({0: dependson})

assert list(ts.static_order()) == [1, 3, 5, 7, 9, 0]
print("TestTopologicalSort::test_graph_with_iterables: ok")
