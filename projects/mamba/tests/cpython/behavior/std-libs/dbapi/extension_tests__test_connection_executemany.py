# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "extension_tests__test_connection_executemany"
# subject = "cpython.test_dbapi.ExtensionTests.test_connection_executemany"
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
con.execute('create table test(foo)')
con.executemany('insert into test(foo) values (?)', [(3,), (4,)])
result = con.execute('select foo from test order by foo').fetchall()
assert result[0][0] == 3
assert result[1][0] == 4

print("ExtensionTests::test_connection_executemany: ok")
