# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_add_subparsers__test_help_extra_prefix_chars"
# subject = "cpython.test_argparse.TestAddSubparsers.test_help_extra_prefix_chars"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestAddSubparsers.test_help_extra_prefix_chars", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestAddSubparsers.test_help_extra_prefix_chars did not pass"
print("TestAddSubparsers::test_help_extra_prefix_chars: ok")
