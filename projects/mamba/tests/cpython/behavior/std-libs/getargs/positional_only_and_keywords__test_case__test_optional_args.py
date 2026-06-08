# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "positional_only_and_keywords__test_case__test_optional_args"
# subject = "cpython.test_getargs.PositionalOnlyAndKeywords_TestCase.test_optional_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_getargs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_getargs
_suite = unittest.defaultTestLoader.loadTestsFromName("PositionalOnlyAndKeywords_TestCase.test_optional_args", test_getargs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PositionalOnlyAndKeywords_TestCase.test_optional_args did not pass"
print("PositionalOnlyAndKeywords_TestCase::test_optional_args: ok")
