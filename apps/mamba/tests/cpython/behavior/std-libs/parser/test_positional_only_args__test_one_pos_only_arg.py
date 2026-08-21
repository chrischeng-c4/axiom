# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parser"
# dimension = "behavior"
# case = "test_positional_only_args__test_one_pos_only_arg"
# subject = "cpython.test_parser.TestPositionalOnlyArgs.test_one_pos_only_arg"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPositionalOnlyArgs.test_one_pos_only_arg", test_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPositionalOnlyArgs.test_one_pos_only_arg did not pass"
print("TestPositionalOnlyArgs::test_one_pos_only_arg: ok")
