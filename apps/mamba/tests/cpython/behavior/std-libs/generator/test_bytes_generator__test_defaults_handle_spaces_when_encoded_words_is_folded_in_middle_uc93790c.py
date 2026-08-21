# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "generator"
# dimension = "behavior"
# case = "test_bytes_generator__test_defaults_handle_spaces_when_encoded_words_is_folded_in_middle_uc93790c"
# subject = "cpython.test_generator.TestBytesGenerator.test_defaults_handle_spaces_when_encoded_words_is_folded_in_middle"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_generator.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_generator
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBytesGenerator.test_defaults_handle_spaces_when_encoded_words_is_folded_in_middle", test_generator)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBytesGenerator.test_defaults_handle_spaces_when_encoded_words_is_folded_in_middle did not pass"
print("TestBytesGenerator::test_defaults_handle_spaces_when_encoded_words_is_folded_in_middle: ok")
