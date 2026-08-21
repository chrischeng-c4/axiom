# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "connection_tests__test_exceptions"
# subject = "cpython.test_dbapi.ConnectionTests.test_exceptions"
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
assert self_cx.Warning == sqlite.Warning
assert self_cx.Error == sqlite.Error
assert self_cx.InterfaceError == sqlite.InterfaceError
assert self_cx.DatabaseError == sqlite.DatabaseError
assert self_cx.DataError == sqlite.DataError
assert self_cx.OperationalError == sqlite.OperationalError
assert self_cx.IntegrityError == sqlite.IntegrityError
assert self_cx.InternalError == sqlite.InternalError
assert self_cx.ProgrammingError == sqlite.ProgrammingError
assert self_cx.NotSupportedError == sqlite.NotSupportedError

print("ConnectionTests::test_exceptions: ok")
