# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "unicode_test__test_issue17223"
# subject = "cpython.test_array.UnicodeTest.test_issue17223"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_array
_suite = unittest.defaultTestLoader.loadTestsFromName("UnicodeTest.test_issue17223", test_array)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnicodeTest.test_issue17223 did not pass"
print("UnicodeTest::test_issue17223: ok")
