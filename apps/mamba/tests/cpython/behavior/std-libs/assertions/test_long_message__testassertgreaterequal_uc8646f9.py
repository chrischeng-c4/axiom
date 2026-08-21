# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "assertions"
# dimension = "behavior"
# case = "test_long_message__testassertgreaterequal_uc8646f9"
# subject = "cpython.test_assertions.TestLongMessage.testAssertGreaterEqual"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_assertions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_assertions
_suite = unittest.defaultTestLoader.loadTestsFromName("TestLongMessage.testAssertGreaterEqual", test_assertions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestLongMessage.testAssertGreaterEqual did not pass"
print("TestLongMessage::testAssertGreaterEqual: ok")
