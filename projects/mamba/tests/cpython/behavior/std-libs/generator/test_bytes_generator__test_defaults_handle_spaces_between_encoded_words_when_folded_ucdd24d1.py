# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "generator"
# dimension = "behavior"
# case = "test_bytes_generator__test_defaults_handle_spaces_between_encoded_words_when_folded_ucdd24d1"
# subject = "cpython.test_generator.TestBytesGenerator.test_defaults_handle_spaces_between_encoded_words_when_folded"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_generator.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_generator
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBytesGenerator.test_defaults_handle_spaces_between_encoded_words_when_folded", test_generator)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBytesGenerator.test_defaults_handle_spaces_between_encoded_words_when_folded did not pass"
print("TestBytesGenerator::test_defaults_handle_spaces_between_encoded_words_when_folded: ok")
