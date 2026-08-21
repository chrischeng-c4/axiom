# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryview"
# dimension = "behavior"
# case = "other_test__test_pickle_ucbb52cb"
# subject = "cpython.test_memoryview.OtherTest.test_pickle"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_memoryview.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_memoryview
_suite = unittest.defaultTestLoader.loadTestsFromName("OtherTest.test_pickle", test_memoryview)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OtherTest.test_pickle did not pass"
print("OtherTest::test_pickle: ok")
