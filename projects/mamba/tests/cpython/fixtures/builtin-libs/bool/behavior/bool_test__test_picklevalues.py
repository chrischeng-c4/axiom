# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_picklevalues"
# subject = "cpython.test.test_bool.BoolTest.test_picklevalues"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_picklevalues
"""Auto-ported test: BoolTest::test_picklevalues (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
import pickle

assert pickle.dumps(True, protocol=0) == b'I01\n.'

assert pickle.dumps(False, protocol=0) == b'I00\n.'

assert pickle.dumps(True, protocol=1) == b'I01\n.'

assert pickle.dumps(False, protocol=1) == b'I00\n.'

assert pickle.dumps(True, protocol=2) == b'\x80\x02\x88.'

assert pickle.dumps(False, protocol=2) == b'\x80\x02\x89.'
print("BoolTest::test_picklevalues: ok")
