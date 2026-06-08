# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileio"
# dimension = "behavior"
# case = "c_other_file_tests__test_bytes_open"
# subject = "cpython.test_fileio.COtherFileTests.testBytesOpen"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileio.py::COtherFileTests::testBytesOpen
"""Auto-ported test: COtherFileTests::testBytesOpen (CPython 3.12 oracle)."""


import sys
import os
import io
import errno
import unittest
from array import array
from weakref import proxy
from functools import wraps
from test.support import cpython_only, swap_attr, gc_collect, is_emscripten, is_wasi
from test.support.os_helper import TESTFN, TESTFN_ASCII, TESTFN_UNICODE, make_bad_fd
from test.support.warnings_helper import check_warnings
from collections import UserList
import _io
import _pyio


def tearDownModule():
    if os.path.exists(TESTFN):
        os.unlink(TESTFN)


# --- test body ---
FileIO = _io.FileIO
modulename = '_io'
fn = TESTFN_ASCII.encode('ascii')
f = FileIO(fn, 'w')
try:
    f.write(b'abc')
    f.close()
    with open(TESTFN_ASCII, 'rb') as f:

        assert f.read() == b'abc'
finally:
    os.unlink(TESTFN_ASCII)
print("COtherFileTests::testBytesOpen: ok")
