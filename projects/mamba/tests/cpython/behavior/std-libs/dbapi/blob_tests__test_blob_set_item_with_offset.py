# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "blob_tests__test_blob_set_item_with_offset"
# subject = "cpython.test_dbapi.BlobTests.test_blob_set_item_with_offset"
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
self_cx.execute('create table test(b blob)')
self_data = b'this blob data string is exactly fifty bytes long!'
self_cx.execute('insert into test(b) values (?)', (self_data,))
self_blob = self_cx.blobopen('test', 'b', 1)
self_blob.seek(0, SEEK_END)
assert self_blob.read() == b''
self_blob[0] = ord('T')
self_blob[-1] = ord('.')
self_blob.seek(0, SEEK_SET)
expected = b'This blob data string is exactly fifty bytes long.'
assert self_blob.read() == expected

print("BlobTests::test_blob_set_item_with_offset: ok")
