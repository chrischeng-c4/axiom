# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "objects"
# dimension = "behavior"
# case = "test_case__test_uc1e6090"
# subject = "cpython.test_objects.TestCase.test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_objects.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_objects
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCase.test", test_objects)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCase.test did not pass"
print("TestCase::test: ok")
