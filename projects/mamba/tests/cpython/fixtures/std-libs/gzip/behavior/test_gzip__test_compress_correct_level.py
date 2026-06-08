# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_gzip__test_compress_correct_level"
# subject = "cpython.test_gzip.TestGzip.test_compress_correct_level"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_gzip.py::TestGzip::test_compress_correct_level
"""Auto-ported test: TestGzip::test_compress_correct_level (CPython 3.12 oracle)."""


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

def write_and_read_back(data, mode='b'):
    b_data = bytes(data)
    with gzip.GzipFile(filename, 'w' + mode) as f:
        l = f.write(data)

    assert l == len(b_data)
    with gzip.GzipFile(filename, 'r' + mode) as f:

        assert f.read() == b_data
os_helper.unlink(filename)
for mtime in (0, 42):
    nocompress = gzip.compress(data1, compresslevel=0, mtime=mtime)
    yescompress = gzip.compress(data1, compresslevel=1, mtime=mtime)

    assert data1 in nocompress

    assert data1 not in yescompress
print("TestGzip::test_compress_correct_level: ok")
