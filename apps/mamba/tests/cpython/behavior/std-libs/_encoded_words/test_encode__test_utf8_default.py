# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_encoded_words"
# dimension = "behavior"
# case = "test_encode__test_utf8_default"
# subject = "cpython.test__encoded_words.TestEncode.test_utf8_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test__encoded_words.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test__encoded_words
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEncode.test_utf8_default", test__encoded_words)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEncode.test_utf8_default did not pass"
print("TestEncode::test_utf8_default: ok")
