# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "sys_settrace"
# dimension = "behavior"
# case = "raising_trace_func_test_case__test_call"
# subject = "cpython.test_sys_settrace.RaisingTraceFuncTestCase.test_call"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys_settrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys_settrace
_suite = unittest.defaultTestLoader.loadTestsFromName("RaisingTraceFuncTestCase.test_call", test_sys_settrace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RaisingTraceFuncTestCase.test_call did not pass"
print("RaisingTraceFuncTestCase::test_call: ok")
