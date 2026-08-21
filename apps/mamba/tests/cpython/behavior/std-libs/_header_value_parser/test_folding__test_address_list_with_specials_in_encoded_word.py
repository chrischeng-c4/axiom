# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_header_value_parser"
# dimension = "behavior"
# case = "test_folding__test_address_list_with_specials_in_encoded_word"
# subject = "cpython.test__header_value_parser.TestFolding.test_address_list_with_specials_in_encoded_word"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test__header_value_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test__header_value_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFolding.test_address_list_with_specials_in_encoded_word", test__header_value_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFolding.test_address_list_with_specials_in_encoded_word did not pass"
print("TestFolding::test_address_list_with_specials_in_encoded_word: ok")
