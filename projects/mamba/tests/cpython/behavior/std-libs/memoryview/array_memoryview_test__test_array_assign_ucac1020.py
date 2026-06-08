# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryview"
# dimension = "behavior"
# case = "array_memoryview_test__test_array_assign_ucac1020"
# subject = "cpython.test_memoryview.ArrayMemoryviewTest.test_array_assign"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_memoryview.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_memoryview
_suite = unittest.defaultTestLoader.loadTestsFromName("ArrayMemoryviewTest.test_array_assign", test_memoryview)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ArrayMemoryviewTest.test_array_assign did not pass"
print("ArrayMemoryviewTest::test_array_assign: ok")
