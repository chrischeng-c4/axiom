# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringptr"
# dimension = "behavior"
# case = "string_ptr_test_case__test__pointer_c_char_uc63aa4b"
# subject = "cpython.test_stringptr.StringPtrTestCase.test__POINTER_c_char"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_stringptr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_stringptr
_suite = unittest.defaultTestLoader.loadTestsFromName("StringPtrTestCase.test__POINTER_c_char", test_stringptr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StringPtrTestCase.test__POINTER_c_char did not pass"
print("StringPtrTestCase::test__POINTER_c_char: ok")
