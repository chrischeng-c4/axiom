# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_gzip__test_flush_flushes_compressor"
# subject = "cpython.test_gzip.TestGzip.test_flush_flushes_compressor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_gzip.py::TestGzip::test_flush_flushes_compressor
"""Auto-ported test: TestGzip::test_flush_flushes_compressor (CPython 3.12 oracle)."""


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
b = io.BytesIO()
message = b'important message here.'
with gzip.GzipFile(fileobj=b, mode='w') as f:
    f.write(message)
    f.flush()
    partial_data = b.getvalue()
full_data = b.getvalue()

assert gzip.decompress(full_data) == message
try:
    gzip.decompress(partial_data)
    raise AssertionError('expected EOFError')
except EOFError:
    pass
d = zlib.decompressobj(wbits=-zlib.MAX_WBITS)
f = io.BytesIO(partial_data)
gzip._read_gzip_header(f)
read_message = d.decompress(f.read())

assert read_message == message
print("TestGzip::test_flush_flushes_compressor: ok")
