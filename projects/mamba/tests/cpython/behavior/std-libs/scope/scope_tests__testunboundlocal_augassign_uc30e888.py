# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "scope"
# dimension = "behavior"
# case = "scope_tests__testunboundlocal_augassign_uc30e888"
# subject = "cpython.test_scope.ScopeTests.testUnboundLocal_AugAssign"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_scope.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_scope
_suite = unittest.defaultTestLoader.loadTestsFromName("ScopeTests.testUnboundLocal_AugAssign", test_scope)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ScopeTests.testUnboundLocal_AugAssign did not pass"
print("ScopeTests::testUnboundLocal_AugAssign: ok")
