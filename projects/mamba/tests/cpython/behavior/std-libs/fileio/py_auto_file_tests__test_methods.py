# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileio"
# dimension = "behavior"
# case = "py_auto_file_tests__test_methods"
# subject = "cpython.test_fileio.PyAutoFileTests.testMethods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileio.py::PyAutoFileTests::testMethods
"""Auto-ported test: PyAutoFileTests::testMethods (CPython 3.12 oracle)."""


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

def ReopenForRead():
    try:
        self_f.close()
    except OSError:
        pass
    self_f = FileIO(TESTFN, 'r')
    os.close(self_f.fileno())
    return self_f

def _testReadintoArray():
    self_f.write(bytes([1, 2, 0, 255]))
    self_f.close()
    a = array('B', b'abcdefgh')
    with FileIO(TESTFN, 'r') as f:
        n = f.readinto(a)

    assert a == array('B', [1, 2, 0, 255, 101, 102, 103, 104])

    assert n == 4
    a = array('b', b'abcdefgh')
    with FileIO(TESTFN, 'r') as f:
        n = f.readinto(a)

    assert a == array('b', [1, 2, 0, -1, 101, 102, 103, 104])

    assert n == 4
    a = array('I', b'abcdefgh')
    with FileIO(TESTFN, 'r') as f:
        n = f.readinto(a)

    assert a == array('I', b'\x01\x02\x00\xffefgh')

    assert n == 4

def _testReadintoMemoryview():
    self_f.write(bytes([1, 2, 0, 255]))
    self_f.close()
    m = memoryview(bytearray(b'abcdefgh'))
    with FileIO(TESTFN, 'r') as f:
        n = f.readinto(m)

    assert m == b'\x01\x02\x00\xffefgh'

    assert n == 4
    m = memoryview(bytearray(b'abcdefgh')).cast('H', shape=[2, 2])
    with FileIO(TESTFN, 'r') as f:
        n = f.readinto(m)

    assert bytes(m) == b'\x01\x02\x00\xffefgh'

    assert n == 4
self_f = FileIO(TESTFN, 'w')
methods = ['fileno', 'isatty', 'seekable', 'readable', 'writable', 'read', 'readall', 'readline', 'readlines', 'tell', 'truncate', 'flush']
self_f.close()

assert self_f.closed
for methodname in methods:
    method = getattr(self_f, methodname)

    try:
        method()
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

try:
    self_f.readinto()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    self_f.readinto(bytearray(1))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    self_f.seek()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    self_f.seek(0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    self_f.write()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    self_f.write(b'')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    self_f.writelines()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    self_f.writelines(b'')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PyAutoFileTests::testMethods: ok")
