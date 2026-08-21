# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_exit_on_error__test_conflicting_mutually_exclusive_args_zero_or_more_with_metavar1"
# subject = "cpython.test_argparse.TestExitOnError.test_conflicting_mutually_exclusive_args_zero_or_more_with_metavar1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestExitOnError.test_conflicting_mutually_exclusive_args_zero_or_more_with_metavar1", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestExitOnError.test_conflicting_mutually_exclusive_args_zero_or_more_with_metavar1 did not pass"
print("TestExitOnError::test_conflicting_mutually_exclusive_args_zero_or_more_with_metavar1: ok")
