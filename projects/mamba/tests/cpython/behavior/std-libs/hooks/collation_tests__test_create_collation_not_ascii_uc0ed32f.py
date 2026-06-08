# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "collation_tests__test_create_collation_not_ascii_uc0ed32f"
# subject = "cpython.test_hooks.CollationTests.test_create_collation_not_ascii"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite
con = sqlite.connect(':memory:')
con.create_collation('collä', lambda x, y: (x > y) - (x < y))

print("CollationTests::test_create_collation_not_ascii: ok")
