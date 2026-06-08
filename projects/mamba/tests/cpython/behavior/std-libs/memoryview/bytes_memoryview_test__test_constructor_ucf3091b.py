# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryview"
# dimension = "behavior"
# case = "bytes_memoryview_test__test_constructor_ucf3091b"
# subject = "cpython.test_memoryview.BytesMemoryviewTest.test_constructor"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_memoryview.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_memoryview
_suite = unittest.defaultTestLoader.loadTestsFromName("BytesMemoryviewTest.test_constructor", test_memoryview)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BytesMemoryviewTest.test_constructor did not pass"
print("BytesMemoryviewTest::test_constructor: ok")
