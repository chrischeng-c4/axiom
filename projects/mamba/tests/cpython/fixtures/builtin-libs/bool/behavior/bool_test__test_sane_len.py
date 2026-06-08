# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_sane_len"
# subject = "cpython.test.test_bool.BoolTest.test_sane_len"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_sane_len
"""Auto-ported test: BoolTest::test_sane_len (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
for badval in ['illegal', -1, 1 << 32]:

    class A:

        def __len__(self):
            return badval
    try:
        bool(A())
    except Exception as e_bool:
        try:
            len(A())
        except Exception as e_len:

            assert str(e_bool) == str(e_len)
print("BoolTest::test_sane_len: ok")
