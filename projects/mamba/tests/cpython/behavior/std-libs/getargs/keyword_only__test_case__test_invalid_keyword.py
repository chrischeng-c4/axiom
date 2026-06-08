# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "keyword_only__test_case__test_invalid_keyword"
# subject = "cpython.test_getargs.KeywordOnly_TestCase.test_invalid_keyword"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_getargs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_getargs
_suite = unittest.defaultTestLoader.loadTestsFromName("KeywordOnly_TestCase.test_invalid_keyword", test_getargs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython KeywordOnly_TestCase.test_invalid_keyword did not pass"
print("KeywordOnly_TestCase::test_invalid_keyword: ok")
