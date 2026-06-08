# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_double_dash__test_subparser_after_multiple_argument_option"
# subject = "cpython.test_argparse.TestDoubleDash.test_subparser_after_multiple_argument_option"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDoubleDash.test_subparser_after_multiple_argument_option", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDoubleDash.test_subparser_after_multiple_argument_option did not pass"
print("TestDoubleDash::test_subparser_after_multiple_argument_option: ok")
