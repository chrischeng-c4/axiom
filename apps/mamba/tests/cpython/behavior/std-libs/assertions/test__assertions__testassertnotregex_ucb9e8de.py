# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "assertions"
# dimension = "behavior"
# case = "test__assertions__testassertnotregex_ucb9e8de"
# subject = "cpython.test_assertions.Test_Assertions.testAssertNotRegex"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_assertions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_assertions
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_Assertions.testAssertNotRegex", test_assertions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_Assertions.testAssertNotRegex did not pass"
print("Test_Assertions::testAssertNotRegex: ok")
