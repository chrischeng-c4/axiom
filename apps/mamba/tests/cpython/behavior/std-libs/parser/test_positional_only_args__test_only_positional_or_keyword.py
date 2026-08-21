# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parser"
# dimension = "behavior"
# case = "test_positional_only_args__test_only_positional_or_keyword"
# subject = "cpython.test_parser.TestPositionalOnlyArgs.test_only_positional_or_keyword"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPositionalOnlyArgs.test_only_positional_or_keyword", test_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPositionalOnlyArgs.test_only_positional_or_keyword did not pass"
print("TestPositionalOnlyArgs::test_only_positional_or_keyword: ok")
