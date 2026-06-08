# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringptr"
# dimension = "behavior"
# case = "string_ptr_test_case__test_functions_uc43802d"
# subject = "cpython.test_stringptr.StringPtrTestCase.test_functions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_stringptr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_stringptr
_suite = unittest.defaultTestLoader.loadTestsFromName("StringPtrTestCase.test_functions", test_stringptr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StringPtrTestCase.test_functions did not pass"
print("StringPtrTestCase::test_functions: ok")
