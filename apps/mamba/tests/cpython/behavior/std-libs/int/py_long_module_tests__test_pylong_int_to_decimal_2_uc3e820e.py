# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int"
# dimension = "behavior"
# case = "py_long_module_tests__test_pylong_int_to_decimal_2_uc3e820e"
# subject = "cpython.test_int.PyLongModuleTests.test_pylong_int_to_decimal_2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_int
_suite = unittest.defaultTestLoader.loadTestsFromName("PyLongModuleTests.test_pylong_int_to_decimal_2", test_int)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyLongModuleTests.test_pylong_int_to_decimal_2 did not pass"
print("PyLongModuleTests::test_pylong_int_to_decimal_2: ok")
