# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct_fields"
# dimension = "behavior"
# case = "struct_fields_test_case__test_gh99275_uc99976e"
# subject = "cpython.test_struct_fields.StructFieldsTestCase.test_gh99275"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_struct_fields.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_struct_fields
_suite = unittest.defaultTestLoader.loadTestsFromName("StructFieldsTestCase.test_gh99275", test_struct_fields)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StructFieldsTestCase.test_gh99275 did not pass"
print("StructFieldsTestCase::test_gh99275: ok")
