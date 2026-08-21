# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_encoded_words"
# dimension = "behavior"
# case = "test_decode_b__test_simple"
# subject = "cpython.test__encoded_words.TestDecodeB.test_simple"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test__encoded_words.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test__encoded_words
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDecodeB.test_simple", test__encoded_words)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDecodeB.test_simple did not pass"
print("TestDecodeB::test_simple: ok")
