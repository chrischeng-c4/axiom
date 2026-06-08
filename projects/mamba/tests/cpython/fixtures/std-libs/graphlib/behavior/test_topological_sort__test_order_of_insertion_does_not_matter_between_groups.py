# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_order_of_insertion_does_not_matter_between_groups"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_order_of_insertion_does_not_matter_between_groups"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_order_of_insertion_does_not_matter_between_groups
"""Auto-ported test: TestTopologicalSort::test_order_of_insertion_does_not_matter_between_groups (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
def get_groups(ts):
    ts.prepare()
    while ts.is_active():
        nodes = ts.get_ready()
        ts.done(*nodes)
        yield set(nodes)
ts = graphlib.TopologicalSorter()
ts.add(3, 2, 1)
ts.add(1, 0)
ts.add(4, 5)
ts.add(6, 7)
ts.add(4, 7)
ts2 = graphlib.TopologicalSorter()
ts2.add(1, 0)
ts2.add(3, 2, 1)
ts2.add(4, 7)
ts2.add(6, 7)
ts2.add(4, 5)

assert list(get_groups(ts)) == list(get_groups(ts2))
print("TestTopologicalSort::test_order_of_insertion_does_not_matter_between_groups: ok")
