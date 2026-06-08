# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_add_dependencies_for_same_node_incrementally"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_add_dependencies_for_same_node_incrementally"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_add_dependencies_for_same_node_incrementally
"""Auto-ported test: TestTopologicalSort::test_add_dependencies_for_same_node_incrementally (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
ts.add(1, 2)
ts.add(1, 3)
ts.add(1, 4)
ts.add(1, 5)
ts2 = graphlib.TopologicalSorter({1: {2, 3, 4, 5}})

assert [*ts.static_order()] == [*ts2.static_order()]
print("TestTopologicalSort::test_add_dependencies_for_same_node_incrementally: ok")
