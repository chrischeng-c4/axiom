# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_mutually_exclusive_group_errors__test_help"
# subject = "cpython.test_argparse.TestMutuallyExclusiveGroupErrors.test_help"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMutuallyExclusiveGroupErrors.test_help", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMutuallyExclusiveGroupErrors.test_help did not pass"
print("TestMutuallyExclusiveGroupErrors::test_help: ok")
