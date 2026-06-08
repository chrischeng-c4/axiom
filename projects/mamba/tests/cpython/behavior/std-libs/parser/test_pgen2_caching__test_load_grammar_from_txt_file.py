# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parser"
# dimension = "behavior"
# case = "test_pgen2_caching__test_load_grammar_from_txt_file"
# subject = "cpython.test_parser.TestPgen2Caching.test_load_grammar_from_txt_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPgen2Caching.test_load_grammar_from_txt_file", test_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPgen2Caching.test_load_grammar_from_txt_file did not pass"
print("TestPgen2Caching::test_load_grammar_from_txt_file: ok")
