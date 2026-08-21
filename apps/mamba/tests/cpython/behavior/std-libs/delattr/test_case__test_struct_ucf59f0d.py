# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "delattr"
# dimension = "behavior"
# case = "test_case__test_struct_ucf59f0d"
# subject = "cpython.test_delattr.TestCase.test_struct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_delattr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_delattr
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCase.test_struct", test_delattr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCase.test_struct did not pass"
print("TestCase::test_struct: ok")
