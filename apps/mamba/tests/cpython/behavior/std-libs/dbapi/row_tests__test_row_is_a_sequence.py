# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "row_tests__test_row_is_a_sequence"
# subject = "cpython.test_dbapi.RowTests.test_row_is_a_sequence"
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
from collections.abc import Sequence
cu = self_cx.execute('SELECT 1')
row = cu.fetchone()
assert issubclass(sqlite.Row, Sequence)
assert isinstance(row, Sequence)

print("RowTests::test_row_is_a_sequence: ok")
