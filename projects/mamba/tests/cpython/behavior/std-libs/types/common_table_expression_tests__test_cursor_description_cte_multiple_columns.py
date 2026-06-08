# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "common_table_expression_tests__test_cursor_description_cte_multiple_columns"
# subject = "cpython.test_types.CommonTableExpressionTests.test_cursor_description_cte_multiple_columns"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import datetime
import sqlite3 as sqlite
import sys
self_con = sqlite.connect(':memory:')
self_cur = self_con.cursor()
self_cur.execute('create table test(x foo)')
self_cur.execute('insert into test values(1)')
self_cur.execute('insert into test values(2)')
self_cur.execute('with testCTE as (select * from test) select * from testCTE')
assert self_cur.description is not None
assert self_cur.description[0][0] == 'x'

print("CommonTableExpressionTests::test_cursor_description_cte_multiple_columns: ok")
