# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_encoded_words"
# dimension = "behavior"
# case = "test_encode_q__test_spaces"
# subject = "cpython.test__encoded_words.TestEncodeQ.test_spaces"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test__encoded_words.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test__encoded_words
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEncodeQ.test_spaces", test__encoded_words)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEncodeQ.test_spaces did not pass"
print("TestEncodeQ::test_spaces: ok")
