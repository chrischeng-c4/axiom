# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audit"
# dimension = "behavior"
# case = "audit_test__test_winapi_createnamedpipe_uc13adb9"
# subject = "cpython.test_audit.AuditTest.test_winapi_createnamedpipe"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audit.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_audit
_suite = unittest.defaultTestLoader.loadTestsFromName("AuditTest.test_winapi_createnamedpipe", test_audit)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AuditTest.test_winapi_createnamedpipe did not pass"
print("AuditTest::test_winapi_createnamedpipe: ok")
