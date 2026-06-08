# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "connection_tests__test_connection_reinit"
# subject = "cpython.test_dbapi.ConnectionTests.test_connection_reinit"
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
cu = self_cx.cursor()
cu.execute('create table test(id integer primary key, name text)')
cu.execute('insert into test(name) values (?)', ('foo',))
db = ':memory:'
cx = sqlite.connect(db)
cx.text_factory = bytes
cx.row_factory = sqlite.Row
cu = cx.cursor()
cu.execute('create table foo (bar)')
cu.executemany('insert into foo (bar) values (?)', ((str(v),) for v in range(4)))
cu.execute('select bar from foo')
rows = [r for r in cu.fetchmany(2)]
assert all((isinstance(r, sqlite.Row) for r in rows))
assert [r[0] for r in rows] == [b'0', b'1']
cx.__init__(db)
cx.execute('create table foo (bar)')
cx.executemany('insert into foo (bar) values (?)', ((v,) for v in ('a', 'b', 'c', 'd')))
rows = [r for r in cu.fetchall()]
assert all((isinstance(r, sqlite.Row) for r in rows))
assert [r[0] for r in rows] == ['2', '3']

print("ConnectionTests::test_connection_reinit: ok")
