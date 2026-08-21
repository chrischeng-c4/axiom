# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "keywords__test_case__test_too_many_args"
# subject = "cpython.test_getargs.Keywords_TestCase.test_too_many_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_getargs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_getargs
_suite = unittest.defaultTestLoader.loadTestsFromName("Keywords_TestCase.test_too_many_args", test_getargs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Keywords_TestCase.test_too_many_args did not pass"
print("Keywords_TestCase::test_too_many_args: ok")
