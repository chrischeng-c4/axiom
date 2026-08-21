# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "sys_settrace"
# dimension = "behavior"
# case = "trace_test_case__test_11_tightloop"
# subject = "cpython.test_sys_settrace.TraceTestCase.test_11_tightloop"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys_settrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys_settrace
_suite = unittest.defaultTestLoader.loadTestsFromName("TraceTestCase.test_11_tightloop", test_sys_settrace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TraceTestCase.test_11_tightloop did not pass"
print("TraceTestCase::test_11_tightloop: ok")
