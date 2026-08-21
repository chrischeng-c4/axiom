# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_24_uc3a2934"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_24"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_named_expressions
_suite = unittest.defaultTestLoader.loadTestsFromName("NamedExpressionScopeTest.test_named_expression_scope_24", test_named_expressions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NamedExpressionScopeTest.test_named_expression_scope_24 did not pass"
print("NamedExpressionScopeTest::test_named_expression_scope_24: ok")
