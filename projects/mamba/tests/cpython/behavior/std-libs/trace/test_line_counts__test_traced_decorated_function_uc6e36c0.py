# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "behavior"
# case = "test_line_counts__test_traced_decorated_function_uc6e36c0"
# subject = "cpython.test_trace.TestLineCounts.test_traced_decorated_function"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_trace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_trace
_suite = unittest.defaultTestLoader.loadTestsFromName("TestLineCounts.test_traced_decorated_function", test_trace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestLineCounts.test_traced_decorated_function did not pass"
print("TestLineCounts::test_traced_decorated_function: ok")
