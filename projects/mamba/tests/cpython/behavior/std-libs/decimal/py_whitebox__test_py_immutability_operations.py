# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "py_whitebox__test_py_immutability_operations"
# subject = "cpython.test_decimal.PyWhitebox.test_py_immutability_operations"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_decimal
_suite = unittest.defaultTestLoader.loadTestsFromName("PyWhitebox.test_py_immutability_operations", test_decimal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyWhitebox.test_py_immutability_operations did not pass"
print("PyWhitebox::test_py_immutability_operations: ok")
