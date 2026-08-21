# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userfunctions"
# dimension = "behavior"
# case = "function_tests__test_func_non_deterministic"
# subject = "cpython.test_userfunctions.FunctionTests.test_func_non_deterministic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_userfunctions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_userfunctions
_suite = unittest.defaultTestLoader.loadTestsFromName("FunctionTests.test_func_non_deterministic", test_userfunctions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FunctionTests.test_func_non_deterministic did not pass"
print("FunctionTests::test_func_non_deterministic: ok")
