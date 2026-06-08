# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "delattr"
# dimension = "behavior"
# case = "test_case__test_chararray_ucc1c4a9"
# subject = "cpython.test_delattr.TestCase.test_chararray"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_delattr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_delattr
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCase.test_chararray", test_delattr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCase.test_chararray did not pass"
print("TestCase::test_chararray: ok")
