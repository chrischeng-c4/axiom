# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_validator_tests__test_if"
# subject = "cpython.test_ast.ASTValidatorTests.test_if"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("ASTValidatorTests.test_if", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ASTValidatorTests.test_if did not pass"
print("ASTValidatorTests::test_if: ok")
