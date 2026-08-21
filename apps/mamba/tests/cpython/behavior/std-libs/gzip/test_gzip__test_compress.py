# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_gzip__test_compress"
# subject = "cpython.test_gzip.TestGzip.test_compress"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_gzip.py::TestGzip::test_compress
"""Auto-ported test: TestGzip::test_compress (CPython 3.12 oracle)."""


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
for data in [data1, data2]:
    for args in [(), (1,), (6,), (9,)]:
        datac = gzip.compress(data, *args)

        assert type(datac) == bytes
        with gzip.GzipFile(fileobj=io.BytesIO(datac), mode='rb') as f:

            assert f.read() == data
print("TestGzip::test_compress: ok")
