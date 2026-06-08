# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryio"
# dimension = "behavior"
# case = "py_bytes_io_test__test_read_no_args"
# subject = "cpython.test_memoryio.PyBytesIOTest.testReadNoArgs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_memoryio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_memoryio.py::PyBytesIOTest::testReadNoArgs
"""Auto-ported test: PyBytesIOTest::testReadNoArgs (CPython 3.12 oracle)."""


import unittest
from test import support
import gc
import io
import _pyio as pyio
import pickle
import sys
import weakref


'Unit tests for memory-based file-like objects.\nStringIO -- for unicode strings\nBytesIO -- for bytes\n'

class IntLike:

    def __init__(self, num):
        self._num = num

    def __index__(self):
        return self._num
    __int__ = __index__


# --- test body ---
UnsupportedOperation = pyio.UnsupportedOperation
ioclass = pyio.BytesIO
EOF = b''

def buftype(s):
    return s.encode('ascii')

def write_ops(f, t):

    assert f.write(t('blah.')) == 5

    assert f.seek(0) == 0

    assert f.write(t('Hello.')) == 6

    assert f.tell() == 6

    assert f.seek(5) == 5

    assert f.tell() == 5

    assert f.write(t(' world\n\n\n')) == 9

    assert f.seek(0) == 0

    assert f.write(t('h')) == 1

    assert f.truncate(12) == 12

    assert f.tell() == 1
buf = buftype('1234567890')
bytesIo = ioclass(buf)

assert buf == bytesIo.read()

assert EOF == bytesIo.read()
print("PyBytesIOTest::testReadNoArgs: ok")
