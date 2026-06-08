# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryio"
# dimension = "behavior"
# case = "py_bytes_io_test__test_readinto"
# subject = "cpython.test_memoryio.PyBytesIOTest.test_readinto"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_memoryio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_memoryio.py::PyBytesIOTest::test_readinto
"""Auto-ported test: PyBytesIOTest::test_readinto (CPython 3.12 oracle)."""


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

class MemorySeekTestMixin:

    def testInit(self):
        buf = self.buftype('1234567890')
        bytesIo = self.ioclass(buf)

    def testRead(self):
        buf = self.buftype('1234567890')
        bytesIo = self.ioclass(buf)
        self.assertEqual(buf[:1], bytesIo.read(1))
        self.assertEqual(buf[1:5], bytesIo.read(4))
        self.assertEqual(buf[5:], bytesIo.read(900))
        self.assertEqual(self.EOF, bytesIo.read())

    def testReadNoArgs(self):
        buf = self.buftype('1234567890')
        bytesIo = self.ioclass(buf)
        self.assertEqual(buf, bytesIo.read())
        self.assertEqual(self.EOF, bytesIo.read())

    def testSeek(self):
        buf = self.buftype('1234567890')
        bytesIo = self.ioclass(buf)
        bytesIo.read(5)
        bytesIo.seek(0)
        self.assertEqual(buf, bytesIo.read())
        bytesIo.seek(3)
        self.assertEqual(buf[3:], bytesIo.read())
        self.assertRaises(TypeError, bytesIo.seek, 0.0)

    def testTell(self):
        buf = self.buftype('1234567890')
        bytesIo = self.ioclass(buf)
        self.assertEqual(0, bytesIo.tell())
        bytesIo.seek(5)
        self.assertEqual(5, bytesIo.tell())
        bytesIo.seek(10000)
        self.assertEqual(10000, bytesIo.tell())


# --- test body ---
UnsupportedOperation = pyio.UnsupportedOperation
ioclass = pyio.BytesIO
EOF = b''

def buftype(s):
    return s.encode('ascii')

def testInit():
    buf = buftype('1234567890')
    bytesIo = ioclass(buf)

def testRead():
    buf = buftype('1234567890')
    bytesIo = ioclass(buf)

    assert buf[:1] == bytesIo.read(1)

    assert buf[1:5] == bytesIo.read(4)

    assert buf[5:] == bytesIo.read(900)

    assert EOF == bytesIo.read()

def testReadNoArgs():
    buf = buftype('1234567890')
    bytesIo = ioclass(buf)

    assert buf == bytesIo.read()

    assert EOF == bytesIo.read()

def testSeek():
    buf = buftype('1234567890')
    bytesIo = ioclass(buf)
    bytesIo.read(5)
    bytesIo.seek(0)

    assert buf == bytesIo.read()
    bytesIo.seek(3)

    assert buf[3:] == bytesIo.read()

    try:
        bytesIo.seek(0.0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def testTell():
    buf = buftype('1234567890')
    bytesIo = ioclass(buf)

    assert 0 == bytesIo.tell()
    bytesIo.seek(5)

    assert 5 == bytesIo.tell()
    bytesIo.seek(10000)

    assert 10000 == bytesIo.tell()

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
memio = ioclass(buf)
b = bytearray(b'hello')

assert memio.readinto(b) == 5

assert b == b'12345'

assert memio.readinto(b) == 5

assert b == b'67890'

assert memio.readinto(b) == 0

assert b == b'67890'
b = bytearray(b'hello world')
memio.seek(0)

assert memio.readinto(b) == 10

assert b == b'1234567890d'
b = bytearray(b'')
memio.seek(0)

assert memio.readinto(b) == 0

assert b == b''

try:
    memio.readinto('')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
import array
a = array.array('b', b'hello world')
memio = ioclass(buf)
memio.readinto(a)

assert a.tobytes() == b'1234567890d'
memio.close()

try:
    memio.readinto(b)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
memio = ioclass(b'123')
b = bytearray()
memio.seek(42)
memio.readinto(b)

assert b == b''
print("PyBytesIOTest::test_readinto: ok")
