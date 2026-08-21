# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_file_type_missing_initialization__test"
# subject = "cpython.test_argparse.TestFileTypeMissingInitialization.test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFileTypeMissingInitialization.test", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFileTypeMissingInitialization.test did not pass"
print("TestFileTypeMissingInitialization::test: ok")
