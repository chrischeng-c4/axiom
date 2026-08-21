# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_trailers_with_redundant_parenthesis"
# subject = "cpython.test_ast.EndPositionTests.test_trailers_with_redundant_parenthesis"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("EndPositionTests.test_trailers_with_redundant_parenthesis", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython EndPositionTests.test_trailers_with_redundant_parenthesis did not pass"
print("EndPositionTests::test_trailers_with_redundant_parenthesis: ok")
