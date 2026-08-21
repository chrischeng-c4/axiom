# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_encoded_words"
# dimension = "behavior"
# case = "test_decode__test_b_undecodable_bytes_ignored_with_defect"
# subject = "cpython.test__encoded_words.TestDecode.test_b_undecodable_bytes_ignored_with_defect"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test__encoded_words.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test__encoded_words
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDecode.test_b_undecodable_bytes_ignored_with_defect", test__encoded_words)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDecode.test_b_undecodable_bytes_ignored_with_defect did not pass"
print("TestDecode::test_b_undecodable_bytes_ignored_with_defect: ok")
