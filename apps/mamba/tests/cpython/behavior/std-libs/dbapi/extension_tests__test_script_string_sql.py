# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "extension_tests__test_script_string_sql"
# subject = "cpython.test_dbapi.ExtensionTests.test_script_string_sql"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import os
import sqlite3 as sqlite
import subprocess
import sys
import threading
import urllib.parse
import warnings
from _testcapi import INT_MAX, ULLONG_MAX
from os import SEEK_SET, SEEK_CUR, SEEK_END
con = sqlite.connect(':memory:')
cur = con.cursor()
cur.executescript('\n            -- bla bla\n            /* a stupid comment */\n            create table a(i);\n            insert into a(i) values (5);\n            ')
cur.execute('select i from a')
res = cur.fetchone()[0]
assert res == 5

print("ExtensionTests::test_script_string_sql: ok")
