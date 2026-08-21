# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "large_array_test__test_fromlist"
# subject = "cpython.test_array.LargeArrayTest.test_fromlist"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_array
_suite = unittest.defaultTestLoader.loadTestsFromName("LargeArrayTest.test_fromlist", test_array)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LargeArrayTest.test_fromlist did not pass"
print("LargeArrayTest::test_fromlist: ok")
