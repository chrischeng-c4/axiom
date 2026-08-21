# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "large_array_test__test_example_data"
# subject = "cpython.test_array.LargeArrayTest.test_example_data"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_array
_suite = unittest.defaultTestLoader.loadTestsFromName("LargeArrayTest.test_example_data", test_array)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LargeArrayTest.test_example_data did not pass"
print("LargeArrayTest::test_example_data: ok")
