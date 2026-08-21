# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "assertions"
# dimension = "behavior"
# case = "test_long_message__test_formatmessage_unicode_error_uca93ad7"
# subject = "cpython.test_assertions.TestLongMessage.test_formatMessage_unicode_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_assertions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_assertions
_suite = unittest.defaultTestLoader.loadTestsFromName("TestLongMessage.test_formatMessage_unicode_error", test_assertions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestLongMessage.test_formatMessage_unicode_error did not pass"
print("TestLongMessage::test_formatMessage_unicode_error: ok")
