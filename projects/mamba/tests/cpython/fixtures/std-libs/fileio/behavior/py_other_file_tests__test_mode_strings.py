# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileio"
# dimension = "behavior"
# case = "py_other_file_tests__test_mode_strings"
# subject = "cpython.test_fileio.PyOtherFileTests.testModeStrings"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileio.py::PyOtherFileTests::testModeStrings
"""Auto-ported test: PyOtherFileTests::testModeStrings (CPython 3.12 oracle)."""


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
    for modes in [('w', 'wb'), ('wb', 'wb'), ('wb+', 'rb+'), ('w+b', 'rb+'), ('a', 'ab'), ('ab', 'ab'), ('ab+', 'ab+'), ('a+b', 'ab+'), ('r', 'rb'), ('rb', 'rb'), ('rb+', 'rb+'), ('r+b', 'rb+')]:
        with FileIO(TESTFN, modes[0]) as f:

            assert f.mode == modes[1]
finally:
    if os.path.exists(TESTFN):
        os.unlink(TESTFN)
print("PyOtherFileTests::testModeStrings: ok")
