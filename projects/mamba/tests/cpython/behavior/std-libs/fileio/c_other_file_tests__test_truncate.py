# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileio"
# dimension = "behavior"
# case = "c_other_file_tests__test_truncate"
# subject = "cpython.test_fileio.COtherFileTests.testTruncate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileio.py::COtherFileTests::testTruncate
"""Auto-ported test: COtherFileTests::testTruncate (CPython 3.12 oracle)."""


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
f = FileIO(TESTFN, 'w')
f.write(bytes(bytearray(range(10))))

assert f.tell() == 10
f.truncate(5)

assert f.tell() == 10

assert f.seek(0, io.SEEK_END) == 5
f.truncate(15)

assert f.tell() == 5

assert f.seek(0, io.SEEK_END) == 15
f.close()
print("COtherFileTests::testTruncate: ok")
