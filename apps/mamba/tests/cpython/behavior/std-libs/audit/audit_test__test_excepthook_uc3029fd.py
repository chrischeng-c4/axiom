# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audit"
# dimension = "behavior"
# case = "audit_test__test_excepthook_uc3029fd"
# subject = "cpython.test_audit.AuditTest.test_excepthook"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audit.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_audit
_suite = unittest.defaultTestLoader.loadTestsFromName("AuditTest.test_excepthook", test_audit)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AuditTest.test_excepthook did not pass"
print("AuditTest::test_excepthook: ok")
