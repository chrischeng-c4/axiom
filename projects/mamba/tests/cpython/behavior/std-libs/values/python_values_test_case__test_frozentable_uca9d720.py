# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "values"
# dimension = "behavior"
# case = "python_values_test_case__test_frozentable_uca9d720"
# subject = "cpython.test_values.PythonValuesTestCase.test_frozentable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_values.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_values
_suite = unittest.defaultTestLoader.loadTestsFromName("PythonValuesTestCase.test_frozentable", test_values)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PythonValuesTestCase.test_frozentable did not pass"
print("PythonValuesTestCase::test_frozentable: ok")
