# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "trace_callback_tests__test_clear_trace_callback_uc2b3ebf"
# subject = "cpython.test_hooks.TraceCallbackTests.test_clear_trace_callback"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite
'\n        Test that setting the trace callback to None clears the previously set callback.\n        '
con = sqlite.connect(':memory:')
traced_statements = []

def trace(statement):
    traced_statements.append(statement)
con.set_trace_callback(trace)
con.set_trace_callback(None)
con.execute('create table foo(a, b)')
assert not traced_statements

print("TraceCallbackTests::test_clear_trace_callback: ok")
