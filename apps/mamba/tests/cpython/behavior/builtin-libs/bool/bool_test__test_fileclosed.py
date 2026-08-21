# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_fileclosed"
# subject = "cpython.test.test_bool.BoolTest.test_fileclosed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_fileclosed
"""Auto-ported test: BoolTest::test_fileclosed (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
try:
    with open(os_helper.TESTFN, 'w', encoding='utf-8') as f:

        assert f.closed is False

    assert f.closed is True
finally:
    os.remove(os_helper.TESTFN)
print("BoolTest::test_fileclosed: ok")
