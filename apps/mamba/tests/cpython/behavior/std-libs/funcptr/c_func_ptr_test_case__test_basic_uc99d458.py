# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "funcptr"
# dimension = "behavior"
# case = "c_func_ptr_test_case__test_basic_uc99d458"
# subject = "cpython.test_funcptr.CFuncPtrTestCase.test_basic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_funcptr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_funcptr
_suite = unittest.defaultTestLoader.loadTestsFromName("CFuncPtrTestCase.test_basic", test_funcptr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CFuncPtrTestCase.test_basic did not pass"
print("CFuncPtrTestCase::test_basic: ok")
