# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "collation_tests__test_create_collation_bad_upper_uc44b936"
# subject = "cpython.test_hooks.CollationTests.test_create_collation_bad_upper"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite

class BadUpperStr(str):

    def upper(self):
        return None
con = sqlite.connect(':memory:')
mycoll = lambda x, y: -((x > y) - (x < y))
con.create_collation(BadUpperStr('mycoll'), mycoll)
result = con.execute("\n            select x from (\n            select 'a' as x\n            union\n            select 'b' as x\n            ) order by x collate mycoll\n            ").fetchall()
assert result[0][0] == 'b'
assert result[1][0] == 'a'

print("CollationTests::test_create_collation_bad_upper: ok")
