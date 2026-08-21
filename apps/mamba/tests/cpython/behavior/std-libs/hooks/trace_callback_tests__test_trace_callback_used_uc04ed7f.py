# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "trace_callback_tests__test_trace_callback_used_uc04ed7f"
# subject = "cpython.test_hooks.TraceCallbackTests.test_trace_callback_used"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite
'\n        Test that the trace callback is invoked once it is set.\n        '
con = sqlite.connect(':memory:')
traced_statements = []

def trace(statement):
    traced_statements.append(statement)
con.set_trace_callback(trace)
con.execute('create table foo(a, b)')
assert traced_statements
assert any(('create table foo' in stmt for stmt in traced_statements))

print("TraceCallbackTests::test_trace_callback_used: ok")
