# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_not_closing_quotes_uca66bb1"
# subject = "cpython.test_fstring.TestCase.test_not_closing_quotes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_fstring
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCase.test_not_closing_quotes", test_fstring)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCase.test_not_closing_quotes did not pass"
print("TestCase::test_not_closing_quotes: ok")
