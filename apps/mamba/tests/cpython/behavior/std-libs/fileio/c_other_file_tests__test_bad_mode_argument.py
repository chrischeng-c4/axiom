# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileio"
# dimension = "behavior"
# case = "c_other_file_tests__test_bad_mode_argument"
# subject = "cpython.test_fileio.COtherFileTests.testBadModeArgument"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileio.py::COtherFileTests::testBadModeArgument
"""Auto-ported test: COtherFileTests::testBadModeArgument (CPython 3.12 oracle)."""


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
bad_mode = 'qwerty'
try:
    f = FileIO(TESTFN, bad_mode)
except ValueError as msg:
    if msg.args[0] != 0:
        s = str(msg)
        if TESTFN in s or bad_mode not in s:

            raise AssertionError('bad error message for invalid mode: %s' % s)
else:
    f.close()

    raise AssertionError('no error for invalid mode: %s' % bad_mode)
print("COtherFileTests::testBadModeArgument: ok")
