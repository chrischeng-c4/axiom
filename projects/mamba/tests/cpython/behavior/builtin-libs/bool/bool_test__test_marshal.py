# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_marshal"
# subject = "cpython.test.test_bool.BoolTest.test_marshal"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_marshal
"""Auto-ported test: BoolTest::test_marshal (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
import marshal

assert marshal.loads(marshal.dumps(True)) is True

assert marshal.loads(marshal.dumps(False)) is False
print("BoolTest::test_marshal: ok")
