# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "collation_tests__test_collation_register_twice_uc451c88"
# subject = "cpython.test_hooks.CollationTests.test_collation_register_twice"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite
'\n        Register two different collation functions under the same name.\n        Verify that the last one is actually used.\n        '
con = sqlite.connect(':memory:')
con.create_collation('mycoll', lambda x, y: (x > y) - (x < y))
con.create_collation('mycoll', lambda x, y: -((x > y) - (x < y)))
result = con.execute("\n            select x from (select 'a' as x union select 'b' as x) order by x collate mycoll\n            ").fetchall()
assert result[0][0] == 'b'
assert result[1][0] == 'a'

print("CollationTests::test_collation_register_twice: ok")
