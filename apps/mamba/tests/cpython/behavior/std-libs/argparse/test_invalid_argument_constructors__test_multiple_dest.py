# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_invalid_argument_constructors__test_multiple_dest"
# subject = "cpython.test_argparse.TestInvalidArgumentConstructors.test_multiple_dest"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInvalidArgumentConstructors.test_multiple_dest", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInvalidArgumentConstructors.test_multiple_dest did not pass"
print("TestInvalidArgumentConstructors::test_multiple_dest: ok")
