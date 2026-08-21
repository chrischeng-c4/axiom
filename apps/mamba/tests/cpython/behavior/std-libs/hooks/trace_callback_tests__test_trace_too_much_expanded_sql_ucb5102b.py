# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "trace_callback_tests__test_trace_too_much_expanded_sql_ucb5102b"
# subject = "cpython.test_hooks.TraceCallbackTests.test_trace_too_much_expanded_sql"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_hooks
_suite = unittest.defaultTestLoader.loadTestsFromName("TraceCallbackTests.test_trace_too_much_expanded_sql", test_hooks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TraceCallbackTests.test_trace_too_much_expanded_sql did not pass"
print("TraceCallbackTests::test_trace_too_much_expanded_sql: ok")
