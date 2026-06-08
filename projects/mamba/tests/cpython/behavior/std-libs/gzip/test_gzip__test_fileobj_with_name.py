# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_gzip__test_fileobj_with_name"
# subject = "cpython.test_gzip.TestGzip.test_fileobj_with_name"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_gzip.py::TestGzip::test_fileobj_with_name
"""Auto-ported test: TestGzip::test_fileobj_with_name (CPython 3.12 oracle)."""


import array
import functools
import gc
import io
import os
import struct
import sys
import unittest
from subprocess import PIPE, Popen
from test.support import catch_unraisable_exception
from test.support import import_helper
from test.support import os_helper
from test.support import _4G, bigmemtest, requires_subprocess
from test.support.script_helper import assert_python_ok, assert_python_failure


'Test script for the gzip module.\n'

gzip = import_helper.import_module('gzip')

zlib = import_helper.import_module('zlib')

data1 = b'  int length=DEFAULTALLOC, err = Z_OK;\n  PyObject *RetVal;\n  int flushmode = Z_FINISH;\n  unsigned long start_total_out;\n\n'

data2 = b'/* zlibmodule.c -- gzip-compatible data compression */\n/* See http://www.gzip.org/zlib/\n/* See http://www.winimage.com/zLibDll for Windows */\n'

TEMPDIR = os.path.abspath(os_helper.TESTFN) + '-gzdir'

class UnseekableIO(io.BytesIO):

    def seekable(self):
        return False

    def tell(self):
        raise io.UnsupportedOperation

    def seek(self, *args):
        raise io.UnsupportedOperation

class BaseTest(unittest.TestCase):
    filename = os_helper.TESTFN

    def setUp(self):
        os_helper.unlink(self.filename)

    def tearDown(self):
        os_helper.unlink(self.filename)

def create_and_remove_directory(directory):

    def decorator(function):

        @functools.wraps(function)
        def wrapper(*args, **kwargs):
            os.makedirs(directory)
            try:
                return function(*args, **kwargs)
            finally:
                os_helper.rmtree(directory)
        return wrapper
    return decorator


# --- test body ---
filename = os_helper.TESTFN
os_helper.unlink(filename)
with open(filename, 'xb') as raw:
    with gzip.GzipFile(fileobj=raw, mode='x') as f:
        f.write(b'one')

        assert f.name == raw.name

        assert f.fileno() == raw.fileno()

        assert f.mode == gzip.WRITE

        assert f.readable() is False

        assert f.writable() is True

        assert f.seekable() is True

        assert f.closed is False

    assert f.closed is True

    assert f.name == raw.name

    try:
        f.fileno()
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass

    assert f.mode == gzip.WRITE

    assert f.readable() is False

    assert f.writable() is True

    assert f.seekable() is True
with open(filename, 'wb') as raw:
    with gzip.GzipFile(fileobj=raw, mode='w') as f:
        f.write(b'two')

        assert f.name == raw.name

        assert f.fileno() == raw.fileno()

        assert f.mode == gzip.WRITE

        assert f.readable() is False

        assert f.writable() is True

        assert f.seekable() is True

        assert f.closed is False

    assert f.closed is True

    assert f.name == raw.name

    try:
        f.fileno()
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass

    assert f.mode == gzip.WRITE

    assert f.readable() is False

    assert f.writable() is True

    assert f.seekable() is True
with open(filename, 'ab') as raw:
    with gzip.GzipFile(fileobj=raw, mode='a') as f:
        f.write(b'three')

        assert f.name == raw.name

        assert f.fileno() == raw.fileno()

        assert f.mode == gzip.WRITE

        assert f.readable() is False

        assert f.writable() is True

        assert f.seekable() is True

        assert f.closed is False

    assert f.closed is True

    assert f.name == raw.name

    try:
        f.fileno()
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass

    assert f.mode == gzip.WRITE

    assert f.readable() is False

    assert f.writable() is True

    assert f.seekable() is True
with open(filename, 'rb') as raw:
    with gzip.GzipFile(fileobj=raw, mode='r') as f:

        assert f.read() == b'twothree'

        assert f.name == raw.name

        assert f.fileno() == raw.fileno()

        assert f.mode == gzip.READ

        assert f.readable() is True

        assert f.writable() is False

        assert f.seekable() is True

        assert f.closed is False

    assert f.closed is True

    assert f.name == raw.name

    try:
        f.fileno()
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass

    assert f.mode == gzip.READ

    assert f.readable() is True

    assert f.writable() is False

    assert f.seekable() is True
print("TestGzip::test_fileobj_with_name: ok")
