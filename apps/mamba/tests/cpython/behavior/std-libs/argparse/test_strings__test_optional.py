# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "test_strings__test_optional"
# subject = "cpython.test_argparse.TestStrings.test_optional"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_argparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestStrings.test_optional", test_argparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestStrings.test_optional did not pass"
print("TestStrings::test_optional: ok")
