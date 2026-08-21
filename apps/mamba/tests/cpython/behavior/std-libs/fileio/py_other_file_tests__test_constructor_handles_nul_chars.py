# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileio"
# dimension = "behavior"
# case = "py_other_file_tests__test_constructor_handles_nul_chars"
# subject = "cpython.test_fileio.PyOtherFileTests.testConstructorHandlesNULChars"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileio.py::PyOtherFileTests::testConstructorHandlesNULChars
"""Auto-ported test: PyOtherFileTests::testConstructorHandlesNULChars (CPython 3.12 oracle)."""


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
fn_with_NUL = 'foo\x00bar'

try:
    FileIO(fn_with_NUL, 'w')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    FileIO(bytes(fn_with_NUL, 'ascii'), 'w')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PyOtherFileTests::testConstructorHandlesNULChars: ok")
