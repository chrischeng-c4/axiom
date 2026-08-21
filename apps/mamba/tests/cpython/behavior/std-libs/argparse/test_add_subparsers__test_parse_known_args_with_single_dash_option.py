# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_add_subparsers__test_parse_known_args_with_single_dash_option"
# subject = "cpython.test_argparse.TestAddSubparsers.test_parse_known_args_with_single_dash_option"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestAddSubparsers.test_parse_known_args_with_single_dash_option", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestAddSubparsers.test_parse_known_args_with_single_dash_option did not pass"
print("TestAddSubparsers::test_parse_known_args_with_single_dash_option: ok")
