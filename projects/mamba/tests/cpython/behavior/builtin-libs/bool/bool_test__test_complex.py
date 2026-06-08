# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_complex"
# subject = "cpython.test.test_bool.BoolTest.test_complex"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_complex
"""Auto-ported test: BoolTest::test_complex (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert complex(False) == 0j

assert complex(False) == False

assert complex(True) == 1 + 0j

assert complex(True) == True
print("BoolTest::test_complex: ok")
