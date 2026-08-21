# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileio"
# dimension = "behavior"
# case = "py_other_file_tests__test_append"
# subject = "cpython.test_fileio.PyOtherFileTests.testAppend"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileio.py::PyOtherFileTests::testAppend
"""Auto-ported test: PyOtherFileTests::testAppend (CPython 3.12 oracle)."""


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
FileIO = _pyio.FileIO
modulename = '_pyio'
try:
    f = open(TESTFN, 'wb')
    f.write(b'spam')
    f.close()
    f = open(TESTFN, 'ab')
    f.write(b'eggs')
    f.close()
    f = open(TESTFN, 'rb')
    d = f.read()
    f.close()

    assert d == b'spameggs'
finally:
    try:
        os.unlink(TESTFN)
    except:
        pass
print("PyOtherFileTests::testAppend: ok")
