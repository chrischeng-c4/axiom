# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "monitoring"
# dimension = "behavior"
# case = "test_regressions__test_call_function_ex"
# subject = "cpython.test_monitoring.TestRegressions.test_call_function_ex"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_monitoring.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_monitoring
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRegressions.test_call_function_ex", test_monitoring)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRegressions.test_call_function_ex did not pass"
print("TestRegressions::test_call_function_ex: ok")
