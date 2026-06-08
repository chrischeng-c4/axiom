# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "test__tokenize__test__tokenize_decodes_with_specified_encoding_uce32aaa"
# subject = "cpython.test_tokenize.Test_Tokenize.test__tokenize_decodes_with_specified_encoding"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tokenize
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_Tokenize.test__tokenize_decodes_with_specified_encoding", test_tokenize)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_Tokenize.test__tokenize_decodes_with_specified_encoding did not pass"
print("Test_Tokenize::test__tokenize_decodes_with_specified_encoding: ok")
