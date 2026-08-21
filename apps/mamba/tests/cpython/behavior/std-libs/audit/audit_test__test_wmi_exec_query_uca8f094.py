# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audit"
# dimension = "behavior"
# case = "audit_test__test_wmi_exec_query_uca8f094"
# subject = "cpython.test_audit.AuditTest.test_wmi_exec_query"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audit.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_audit
_suite = unittest.defaultTestLoader.loadTestsFromName("AuditTest.test_wmi_exec_query", test_audit)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AuditTest.test_wmi_exec_query did not pass"
print("AuditTest::test_wmi_exec_query: ok")
