# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "behavior"
# case = "test_callers__test_loop_caller_importing_uc1f4004"
# subject = "cpython.test_trace.TestCallers.test_loop_caller_importing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_trace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_trace
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCallers.test_loop_caller_importing", test_trace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCallers.test_loop_caller_importing did not pass"
print("TestCallers::test_loop_caller_importing: ok")
