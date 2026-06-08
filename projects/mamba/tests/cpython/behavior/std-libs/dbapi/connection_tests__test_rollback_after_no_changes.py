# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "connection_tests__test_rollback_after_no_changes"
# subject = "cpython.test_dbapi.ConnectionTests.test_rollback_after_no_changes"
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
'\n        A rollback should also work when no changes were made to the database.\n        '
self_cx.rollback()
self_cx.rollback()

print("ConnectionTests::test_rollback_after_no_changes: ok")
