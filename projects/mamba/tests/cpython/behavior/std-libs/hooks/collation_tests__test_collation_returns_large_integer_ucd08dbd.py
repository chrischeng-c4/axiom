# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "collation_tests__test_collation_returns_large_integer_ucd08dbd"
# subject = "cpython.test_hooks.CollationTests.test_collation_returns_large_integer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite

def mycoll(x, y):
    return -((x > y) - (x < y)) * 2 ** 32
con = sqlite.connect(':memory:')
con.create_collation('mycoll', mycoll)
sql = "\n            select x from (\n            select 'a' as x\n            union\n            select 'b' as x\n            union\n            select 'c' as x\n            ) order by x collate mycoll\n            "
result = con.execute(sql).fetchall()
assert result == [('c',), ('b',), ('a',)]

print("CollationTests::test_collation_returns_large_integer: ok")
