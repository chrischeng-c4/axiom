# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_header_value_parser"
# dimension = "behavior"
# case = "test_folding__test_split_at_whitespace_after_header_before_long_token"
# subject = "cpython.test__header_value_parser.TestFolding.test_split_at_whitespace_after_header_before_long_token"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test__header_value_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test__header_value_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFolding.test_split_at_whitespace_after_header_before_long_token", test__header_value_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFolding.test_split_at_whitespace_after_header_before_long_token did not pass"
print("TestFolding::test_split_at_whitespace_after_header_before_long_token: ok")
