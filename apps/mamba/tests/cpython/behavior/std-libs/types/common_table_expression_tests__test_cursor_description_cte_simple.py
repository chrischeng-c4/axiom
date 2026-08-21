# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "common_table_expression_tests__test_cursor_description_cte_simple"
# subject = "cpython.test_types.CommonTableExpressionTests.test_cursor_description_cte_simple"
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
self_cur.execute('with one as (select 1) select * from one')
assert self_cur.description is not None
assert self_cur.description[0][0] == '1'

print("CommonTableExpressionTests::test_cursor_description_cte_simple: ok")
