# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__testunoptimizednamespaces_uc883957"
# subject = "cpython.test_scope.ScopeTests.testUnoptimizedNamespaces"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_scope
_suite = unittest.defaultTestLoader.loadTestsFromName("ScopeTests.testUnoptimizedNamespaces", test_scope)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ScopeTests.testUnoptimizedNamespaces did not pass"
print("ScopeTests::testUnoptimizedNamespaces: ok")
