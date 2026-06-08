# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "behavior"
# case = "test_multiple_args_append__test_nargs_append_simple"
# subject = "cpython.test_optparse.TestMultipleArgsAppend.test_nargs_append_simple"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_optparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_optparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMultipleArgsAppend.test_nargs_append_simple", test_optparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMultipleArgsAppend.test_nargs_append_simple did not pass"
print("TestMultipleArgsAppend::test_nargs_append_simple: ok")
