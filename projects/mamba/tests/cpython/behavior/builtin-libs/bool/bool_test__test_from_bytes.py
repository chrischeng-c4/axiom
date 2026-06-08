# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_from_bytes"
# subject = "cpython.test.test_bool.BoolTest.test_from_bytes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_from_bytes
"""Auto-ported test: BoolTest::test_from_bytes (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert bool.from_bytes(b'\x00' * 8, 'big') is False

assert bool.from_bytes(b'abcd', 'little') is True
print("BoolTest::test_from_bytes: ok")
