# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "positional_only_arg"
# dimension = "behavior"
# case = "positional_only_test_case__test_invalid_syntax_lambda_ucc4eb9c"
# subject = "cpython.test_positional_only_arg.PositionalOnlyTestCase.test_invalid_syntax_lambda"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_positional_only_arg.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_positional_only_arg
_suite = unittest.defaultTestLoader.loadTestsFromName("PositionalOnlyTestCase.test_invalid_syntax_lambda", test_positional_only_arg)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PositionalOnlyTestCase.test_invalid_syntax_lambda did not pass"
print("PositionalOnlyTestCase::test_invalid_syntax_lambda: ok")
