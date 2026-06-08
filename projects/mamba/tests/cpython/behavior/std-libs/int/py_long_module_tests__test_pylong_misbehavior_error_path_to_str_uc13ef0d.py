# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int"
# dimension = "behavior"
# case = "py_long_module_tests__test_pylong_misbehavior_error_path_to_str_uc13ef0d"
# subject = "cpython.test_int.PyLongModuleTests.test_pylong_misbehavior_error_path_to_str"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_int
_suite = unittest.defaultTestLoader.loadTestsFromName("PyLongModuleTests.test_pylong_misbehavior_error_path_to_str", test_int)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyLongModuleTests.test_pylong_misbehavior_error_path_to_str did not pass"
print("PyLongModuleTests::test_pylong_misbehavior_error_path_to_str: ok")
