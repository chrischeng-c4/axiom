# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "large_array_test__test_append"
# subject = "cpython.test_array.LargeArrayTest.test_append"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_array
_suite = unittest.defaultTestLoader.loadTestsFromName("LargeArrayTest.test_append", test_array)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LargeArrayTest.test_append did not pass"
print("LargeArrayTest::test_append: ok")
