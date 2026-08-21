# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_backslashes_in_expression_part_uc8f618c"
# subject = "cpython.test_fstring.TestCase.test_backslashes_in_expression_part"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_fstring
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCase.test_backslashes_in_expression_part", test_fstring)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCase.test_backslashes_in_expression_part did not pass"
print("TestCase::test_backslashes_in_expression_part: ok")
