# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_file_type_open_args__test_open_args"
# subject = "cpython.test_argparse.TestFileTypeOpenArgs.test_open_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFileTypeOpenArgs.test_open_args", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFileTypeOpenArgs.test_open_args did not pass"
print("TestFileTypeOpenArgs::test_open_args: ok")
