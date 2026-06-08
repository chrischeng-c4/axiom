# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "cursor_tests__test_execute_arg_string_with_zero_byte"
# subject = "cpython.test_dbapi.CursorTests.test_execute_arg_string_with_zero_byte"
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
self_cu.execute('create table test(id integer primary key, name text, income number, unique_test text unique)')
self_cu.execute('insert into test(name) values (?)', ('foo',))
self_cu.execute('insert into test(name) values (?)', ('Hu\x00go',))
self_cu.execute('select name from test where id=?', (self_cu.lastrowid,))
row = self_cu.fetchone()
assert row[0] == 'Hu\x00go'

print("CursorTests::test_execute_arg_string_with_zero_byte: ok")
