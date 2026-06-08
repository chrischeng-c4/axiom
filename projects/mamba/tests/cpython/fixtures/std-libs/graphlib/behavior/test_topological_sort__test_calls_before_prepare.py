# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_calls_before_prepare"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_calls_before_prepare"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_calls_before_prepare
"""Auto-ported test: TestTopologicalSort::test_calls_before_prepare (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
ts = graphlib.TopologicalSorter()
try:
    ts.get_ready()
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('prepare\\(\\) must be called first', str(_aR_e))
try:
    ts.done(3)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('prepare\\(\\) must be called first', str(_aR_e))
try:
    ts.is_active()
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('prepare\\(\\) must be called first', str(_aR_e))
print("TestTopologicalSort::test_calls_before_prepare: ok")
