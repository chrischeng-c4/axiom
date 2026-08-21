# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryview"
# dimension = "behavior"
# case = "other_test__test_copy_uc32479d"
# subject = "cpython.test_memoryview.OtherTest.test_copy"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_memoryview.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_memoryview
_suite = unittest.defaultTestLoader.loadTestsFromName("OtherTest.test_copy", test_memoryview)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OtherTest.test_copy did not pass"
print("OtherTest::test_copy: ok")
