use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/memoryio/c_bytes_io_test__test_read_no_args.py`.
#[test]
fn test_gen_behavior_std_libs_memoryio_c_bytes_io_test__test_read_no_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryio"
# dimension = "behavior"
# case = "c_bytes_io_test__test_read_no_args"
# subject = "cpython.test_memoryio.CBytesIOTest.testReadNoArgs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_memoryio.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_memoryio.py::CBytesIOTest::testReadNoArgs
"""Auto-ported test: CBytesIOTest::testReadNoArgs (CPython 3.12 oracle)."""


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
ioclass = io.BytesIO
UnsupportedOperation = io.UnsupportedOperation
check_sizeof = support.check_sizeof

def _test_cow_mutation(mutation):
    imm = b' ' * 1024
    old_rc = sys.getrefcount(imm)
    memio = ioclass(imm)

    assert sys.getrefcount(imm) == old_rc + 1
    mutation(memio)

    assert sys.getrefcount(imm) == old_rc

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
print("CBytesIOTest::testReadNoArgs: ok")
"###);
    assert_output(&out, r###"CBytesIOTest::testReadNoArgs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/memoryio/c_string_io_test__test_read_no_args.py`.
#[test]
fn test_gen_behavior_std_libs_memoryio_c_string_io_test__test_read_no_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryio"
# dimension = "behavior"
# case = "c_string_io_test__test_read_no_args"
# subject = "cpython.test_memoryio.CStringIOTest.testReadNoArgs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_memoryio.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_memoryio.py::CStringIOTest::testReadNoArgs
"""Auto-ported test: CStringIOTest::testReadNoArgs (CPython 3.12 oracle)."""


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
buftype = str
ioclass = pyio.StringIO
UnsupportedOperation = pyio.UnsupportedOperation
EOF = ''
ioclass = io.StringIO
UnsupportedOperation = io.UnsupportedOperation

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
print("CStringIOTest::testReadNoArgs: ok")
"###);
    assert_output(&out, r###"CStringIOTest::testReadNoArgs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/memoryio/py_bytes_io_test__test_instance_dict_leak.py`.
#[test]
fn test_gen_behavior_std_libs_memoryio_py_bytes_io_test__test_instance_dict_leak() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryio"
# dimension = "behavior"
# case = "py_bytes_io_test__test_instance_dict_leak"
# subject = "cpython.test_memoryio.PyBytesIOTest.test_instance_dict_leak"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_memoryio.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_memoryio.py::PyBytesIOTest::test_instance_dict_leak
"""Auto-ported test: PyBytesIOTest::test_instance_dict_leak (CPython 3.12 oracle)."""


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
for _ in range(100):
    memio = ioclass()
    memio.foo = 1
print("PyBytesIOTest::test_instance_dict_leak: ok")
"###);
    assert_output(&out, r###"PyBytesIOTest::test_instance_dict_leak: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/memoryio/py_string_io_test__test_instance_dict_leak.py`.
#[test]
fn test_gen_behavior_std_libs_memoryio_py_string_io_test__test_instance_dict_leak() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryio"
# dimension = "behavior"
# case = "py_string_io_test__test_instance_dict_leak"
# subject = "cpython.test_memoryio.PyStringIOTest.test_instance_dict_leak"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_memoryio.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_memoryio.py::PyStringIOTest::test_instance_dict_leak
"""Auto-ported test: PyStringIOTest::test_instance_dict_leak (CPython 3.12 oracle)."""


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
buftype = str
ioclass = pyio.StringIO
UnsupportedOperation = pyio.UnsupportedOperation
EOF = ''

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
for _ in range(100):
    memio = ioclass()
    memio.foo = 1
print("PyStringIOTest::test_instance_dict_leak: ok")
"###);
    assert_output(&out, r###"PyStringIOTest::test_instance_dict_leak: ok
"###);
}
