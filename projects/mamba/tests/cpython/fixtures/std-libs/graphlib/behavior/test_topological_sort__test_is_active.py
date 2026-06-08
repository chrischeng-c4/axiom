# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_is_active"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_is_active"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_is_active
"""Auto-ported test: TestTopologicalSort::test_is_active (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
ts.add(1, 2)
ts.prepare()

assert ts.is_active()

assert ts.get_ready() == (2,)

assert ts.is_active()
ts.done(2)

assert ts.is_active()

assert ts.get_ready() == (1,)

assert ts.is_active()
ts.done(1)

assert not ts.is_active()
print("TestTopologicalSort::test_is_active: ok")
