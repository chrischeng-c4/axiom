# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "row_tests__test_row_equality"
# subject = "cpython.test_dbapi.RowTests.test_row_equality"
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
self_cx.row_factory = sqlite.Row
c1 = self_cx.execute('SELECT 1 as a')
r1 = c1.fetchone()
c2 = self_cx.execute('SELECT 1 as a')
r2 = c2.fetchone()
assert r1 is not r2
assert r1 == r2
c3 = self_cx.execute('SELECT 1 as b')
r3 = c3.fetchone()
assert r1 != r3

print("RowTests::test_row_equality: ok")
