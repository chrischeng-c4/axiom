# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_done"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_done"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_done
"""Auto-ported test: TestTopologicalSort::test_done (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
ts.add(1, 2, 3, 4)
ts.add(2, 3)
ts.prepare()

assert ts.get_ready() == (3, 4)

assert ts.get_ready() == ()
ts.done(3)

assert ts.get_ready() == (2,)

assert ts.get_ready() == ()
ts.done(4)
ts.done(2)

assert ts.get_ready() == (1,)

assert ts.get_ready() == ()
ts.done(1)

assert ts.get_ready() == ()

assert not ts.is_active()
print("TestTopologicalSort::test_done: ok")
