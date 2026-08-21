# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "module_state_tests__test_subinterpreter"
# subject = "cpython.test_ast.ModuleStateTests.test_subinterpreter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ast import test_ast
_suite = unittest.defaultTestLoader.loadTestsFromName("ModuleStateTests.test_subinterpreter", test_ast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ModuleStateTests.test_subinterpreter did not pass"
print("ModuleStateTests::test_subinterpreter: ok")
