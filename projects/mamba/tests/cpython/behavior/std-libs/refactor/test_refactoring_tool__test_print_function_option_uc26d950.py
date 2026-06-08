# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "refactor"
# dimension = "behavior"
# case = "test_refactoring_tool__test_print_function_option_uc26d950"
# subject = "cpython.test_refactor.TestRefactoringTool.test_print_function_option"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_refactor.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_refactor
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRefactoringTool.test_print_function_option", test_refactor)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRefactoringTool.test_print_function_option did not pass"
print("TestRefactoringTool::test_print_function_option: ok")
