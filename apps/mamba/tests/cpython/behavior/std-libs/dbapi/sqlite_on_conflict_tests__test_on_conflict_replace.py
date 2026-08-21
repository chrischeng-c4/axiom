# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "sqlite_on_conflict_tests__test_on_conflict_replace"
# subject = "cpython.test_dbapi.SqliteOnConflictTests.test_on_conflict_replace"
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
self_cx = sqlite.connect(':memory:')
self_cu = self_cx.cursor()
self_cu.execute('\n          CREATE TABLE test(\n            id INTEGER PRIMARY KEY, name TEXT, unique_name TEXT UNIQUE\n          );\n        ')
self_cu.execute("INSERT OR REPLACE INTO test(name, unique_name) VALUES ('Data!', 'foo')")
self_cu.execute("INSERT OR REPLACE INTO test(name, unique_name) VALUES ('Very different data!', 'foo')")
self_cu.execute('SELECT name, unique_name FROM test')
assert self_cu.fetchall() == [('Very different data!', 'foo')]

print("SqliteOnConflictTests::test_on_conflict_replace: ok")
