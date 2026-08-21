# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_other_file_tests__test_truncate_on_windows"
# subject = "cpython.test_file.COtherFileTests.testTruncateOnWindows"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::COtherFileTests::testTruncateOnWindows
"""Auto-ported test: COtherFileTests::testTruncateOnWindows (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = io.open

def _checkBufferSize(s):
    try:
        f = open(TESTFN, 'wb', s)
        f.write(str(s).encode('ascii'))
        f.close()
        f.close()
        f = open(TESTFN, 'rb', s)
        d = int(f.read().decode('ascii'))
        f.close()
        f.close()
    except OSError as msg:

        raise AssertionError('error setting buffer size %d: %s' % (s, str(msg)))

    assert d == s
f = open(TESTFN, 'wb')
try:
    f.write(b'12345678901')
    f.close()
    f = open(TESTFN, 'rb+')
    data = f.read(5)
    if data != b'12345':

        raise AssertionError('Read on file opened for update failed %r' % data)
    if f.tell() != 5:

        raise AssertionError('File pos after read wrong %d' % f.tell())
    f.truncate()
    if f.tell() != 5:

        raise AssertionError('File pos after ftruncate wrong %d' % f.tell())
    f.close()
    size = os.path.getsize(TESTFN)
    if size != 5:

        raise AssertionError('File size after ftruncate wrong %d' % size)
finally:
    f.close()
print("COtherFileTests::testTruncateOnWindows: ok")
