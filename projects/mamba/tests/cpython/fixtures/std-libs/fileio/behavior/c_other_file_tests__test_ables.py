# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileio"
# dimension = "behavior"
# case = "c_other_file_tests__test_ables"
# subject = "cpython.test_fileio.COtherFileTests.testAbles"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileio.py::COtherFileTests::testAbles
"""Auto-ported test: COtherFileTests::testAbles (CPython 3.12 oracle)."""


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
try:
    f = FileIO(TESTFN, 'w')

    assert f.readable() == False

    assert f.writable() == True

    assert f.seekable() == True
    f.close()
    f = FileIO(TESTFN, 'r')

    assert f.readable() == True

    assert f.writable() == False

    assert f.seekable() == True
    f.close()
    f = FileIO(TESTFN, 'a+')

    assert f.readable() == True

    assert f.writable() == True

    assert f.seekable() == True

    assert f.isatty() == False
    f.close()
    if sys.platform != 'win32' and (not is_emscripten):
        try:
            f = FileIO('/dev/tty', 'a')
        except OSError:
            pass
        else:

            assert f.readable() == False

            assert f.writable() == True
            if sys.platform != 'darwin' and 'bsd' not in sys.platform and (not sys.platform.startswith(('sunos', 'aix'))):

                assert f.seekable() == False

            assert f.isatty() == True
            f.close()
finally:
    os.unlink(TESTFN)
print("COtherFileTests::testAbles: ok")
