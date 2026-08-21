# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "extension_tests__test_cursor_executescript_tx_control"
# subject = "cpython.test_dbapi.ExtensionTests.test_cursor_executescript_tx_control"
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
con.execute('begin')
assert con.in_transaction
con.executescript('select 1')
assert not con.in_transaction

print("ExtensionTests::test_cursor_executescript_tx_control: ok")
