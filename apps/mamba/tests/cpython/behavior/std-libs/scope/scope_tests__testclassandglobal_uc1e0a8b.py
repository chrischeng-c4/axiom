# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__testclassandglobal_uc1e0a8b"
# subject = "cpython.test_scope.ScopeTests.testClassAndGlobal"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_scope
_suite = unittest.defaultTestLoader.loadTestsFromName("ScopeTests.testClassAndGlobal", test_scope)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ScopeTests.testClassAndGlobal did not pass"
print("ScopeTests::testClassAndGlobal: ok")
