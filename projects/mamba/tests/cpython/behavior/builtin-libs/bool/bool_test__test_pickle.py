# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_pickle"
# subject = "cpython.test.test_bool.BoolTest.test_pickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_pickle
"""Auto-ported test: BoolTest::test_pickle (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
import pickle
for proto in range(pickle.HIGHEST_PROTOCOL + 1):

    assert pickle.loads(pickle.dumps(True, proto)) is True

    assert pickle.loads(pickle.dumps(False, proto)) is False
print("BoolTest::test_pickle: ok")
