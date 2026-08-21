# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "refactor"
# dimension = "behavior"
# case = "test_refactoring_tool__test_explicit_uce26539"
# subject = "cpython.test_refactor.TestRefactoringTool.test_explicit"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_refactor.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_refactor
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRefactoringTool.test_explicit", test_refactor)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRefactoringTool.test_explicit did not pass"
print("TestRefactoringTool::test_explicit: ok")
