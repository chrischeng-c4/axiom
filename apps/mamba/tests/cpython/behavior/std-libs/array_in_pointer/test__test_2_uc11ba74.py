# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array_in_pointer"
# dimension = "behavior"
# case = "test__test_2_uc11ba74"
# subject = "cpython.test_array_in_pointer.Test.test_2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_array_in_pointer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_array_in_pointer
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_2", test_array_in_pointer)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_2 did not pass"
print("Test::test_2: ok")
