# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_add_argument_metavar__test_nargs_parser_metavar_length2"
# subject = "cpython.test_argparse.TestAddArgumentMetavar.test_nargs_parser_metavar_length2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestAddArgumentMetavar.test_nargs_parser_metavar_length2", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestAddArgumentMetavar.test_nargs_parser_metavar_length2 did not pass"
print("TestAddArgumentMetavar::test_nargs_parser_metavar_length2: ok")
