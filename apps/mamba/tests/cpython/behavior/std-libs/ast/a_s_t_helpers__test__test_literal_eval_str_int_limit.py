# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_literal_eval_str_int_limit"
# subject = "cpython.test_ast.ASTHelpers_Test.test_literal_eval_str_int_limit"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("ASTHelpers_Test.test_literal_eval_str_int_limit", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ASTHelpers_Test.test_literal_eval_str_int_limit did not pass"
print("ASTHelpers_Test::test_literal_eval_str_int_limit: ok")
