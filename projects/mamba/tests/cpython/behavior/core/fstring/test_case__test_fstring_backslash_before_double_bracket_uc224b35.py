# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_fstring_backslash_before_double_bracket_uc224b35"
# subject = "cpython.test_fstring.TestCase.test_fstring_backslash_before_double_bracket"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_fstring
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCase.test_fstring_backslash_before_double_bracket", test_fstring)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCase.test_fstring_backslash_before_double_bracket did not pass"
print("TestCase::test_fstring_backslash_before_double_bracket: ok")
