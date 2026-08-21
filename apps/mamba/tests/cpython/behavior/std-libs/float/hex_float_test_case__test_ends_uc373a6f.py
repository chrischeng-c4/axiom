# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "float"
# dimension = "behavior"
# case = "hex_float_test_case__test_ends_uc373a6f"
# subject = "cpython.test_float.HexFloatTestCase.test_ends"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_float
_suite = unittest.defaultTestLoader.loadTestsFromName("HexFloatTestCase.test_ends", test_float)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HexFloatTestCase.test_ends did not pass"
print("HexFloatTestCase::test_ends: ok")
