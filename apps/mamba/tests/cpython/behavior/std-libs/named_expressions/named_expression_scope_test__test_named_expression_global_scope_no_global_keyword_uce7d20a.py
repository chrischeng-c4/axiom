# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_global_scope_no_global_keyword_uce7d20a"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_global_scope_no_global_keyword"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_named_expressions
_suite = unittest.defaultTestLoader.loadTestsFromName("NamedExpressionScopeTest.test_named_expression_global_scope_no_global_keyword", test_named_expressions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NamedExpressionScopeTest.test_named_expression_global_scope_no_global_keyword did not pass"
print("NamedExpressionScopeTest::test_named_expression_global_scope_no_global_keyword: ok")
