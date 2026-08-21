# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "trace_callback_tests__test_unicode_content_ucb5c529"
# subject = "cpython.test_hooks.TraceCallbackTests.test_unicode_content"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite
'\n        Test that the statement can contain unicode literals.\n        '
unicode_value = 'öäüÖÄÜß€'
con = sqlite.connect(':memory:')
traced_statements = []

def trace(statement):
    traced_statements.append(statement)
con.set_trace_callback(trace)
con.execute('create table foo(x)')
con.execute("insert into foo(x) values ('%s')" % unicode_value)
con.commit()
assert any((unicode_value in stmt for stmt in traced_statements))

print("TraceCallbackTests::test_unicode_content: ok")
