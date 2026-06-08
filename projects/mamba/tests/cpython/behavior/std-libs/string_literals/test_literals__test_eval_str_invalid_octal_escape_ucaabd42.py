# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string_literals"
# dimension = "behavior"
# case = "test_literals__test_eval_str_invalid_octal_escape_ucaabd42"
# subject = "cpython.test_string_literals.TestLiterals.test_eval_str_invalid_octal_escape"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string_literals.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_string_literals
_suite = unittest.defaultTestLoader.loadTestsFromName("TestLiterals.test_eval_str_invalid_octal_escape", test_string_literals)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestLiterals.test_eval_str_invalid_octal_escape did not pass"
print("TestLiterals::test_eval_str_invalid_octal_escape: ok")
