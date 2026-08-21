# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileio"
# dimension = "behavior"
# case = "c_auto_file_tests__test_none_args"
# subject = "cpython.test_fileio.CAutoFileTests.test_none_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileio.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileio.py::CAutoFileTests::test_none_args
"""Auto-ported test: CAutoFileTests::test_none_args (CPython 3.12 oracle)."""


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


class OtherFileTests:

    def testAbles(self):
        try:
            f = self.FileIO(TESTFN, 'w')
            self.assertEqual(f.readable(), False)
            self.assertEqual(f.writable(), True)
            self.assertEqual(f.seekable(), True)
            f.close()
            f = self.FileIO(TESTFN, 'r')
            self.assertEqual(f.readable(), True)
            self.assertEqual(f.writable(), False)
            self.assertEqual(f.seekable(), True)
            f.close()
            f = self.FileIO(TESTFN, 'a+')
            self.assertEqual(f.readable(), True)
            self.assertEqual(f.writable(), True)
            self.assertEqual(f.seekable(), True)
            self.assertEqual(f.isatty(), False)
            f.close()
            if sys.platform != 'win32' and (not is_emscripten):
                try:
                    f = self.FileIO('/dev/tty', 'a')
                except OSError:
                    pass
                else:
                    self.assertEqual(f.readable(), False)
                    self.assertEqual(f.writable(), True)
                    if sys.platform != 'darwin' and 'bsd' not in sys.platform and (not sys.platform.startswith(('sunos', 'aix'))):
                        self.assertEqual(f.seekable(), False)
                    self.assertEqual(f.isatty(), True)
                    f.close()
        finally:
            os.unlink(TESTFN)

    def testInvalidModeStrings(self):
        for mode in ('', 'aU', 'wU+', 'rw', 'rt'):
            try:
                f = self.FileIO(TESTFN, mode)
            except ValueError:
                pass
            else:
                f.close()
                self.fail('%r is an invalid file mode' % mode)

    def testModeStrings(self):
        try:
            for modes in [('w', 'wb'), ('wb', 'wb'), ('wb+', 'rb+'), ('w+b', 'rb+'), ('a', 'ab'), ('ab', 'ab'), ('ab+', 'ab+'), ('a+b', 'ab+'), ('r', 'rb'), ('rb', 'rb'), ('rb+', 'rb+'), ('r+b', 'rb+')]:
                with self.FileIO(TESTFN, modes[0]) as f:
                    self.assertEqual(f.mode, modes[1])
        finally:
            if os.path.exists(TESTFN):
                os.unlink(TESTFN)

    def testUnicodeOpen(self):
        f = self.FileIO(str(TESTFN), 'w')
        f.close()
        os.unlink(TESTFN)

    def testBytesOpen(self):
        fn = TESTFN_ASCII.encode('ascii')
        f = self.FileIO(fn, 'w')
        try:
            f.write(b'abc')
            f.close()
            with open(TESTFN_ASCII, 'rb') as f:
                self.assertEqual(f.read(), b'abc')
        finally:
            os.unlink(TESTFN_ASCII)

    @unittest.skipIf(sys.getfilesystemencoding() != 'utf-8', 'test only works for utf-8 filesystems')
    def testUtf8BytesOpen(self):
        try:
            fn = TESTFN_UNICODE.encode('utf-8')
        except UnicodeEncodeError:
            self.skipTest('could not encode %r to utf-8' % TESTFN_UNICODE)
        f = self.FileIO(fn, 'w')
        try:
            f.write(b'abc')
            f.close()
            with open(TESTFN_UNICODE, 'rb') as f:
                self.assertEqual(f.read(), b'abc')
        finally:
            os.unlink(TESTFN_UNICODE)

    def testConstructorHandlesNULChars(self):
        fn_with_NUL = 'foo\x00bar'
        self.assertRaises(ValueError, self.FileIO, fn_with_NUL, 'w')
        self.assertRaises(ValueError, self.FileIO, bytes(fn_with_NUL, 'ascii'), 'w')

    def testInvalidFd(self):
        self.assertRaises(ValueError, self.FileIO, -10)
        self.assertRaises(OSError, self.FileIO, make_bad_fd())
        if sys.platform == 'win32':
            import msvcrt
            self.assertRaises(OSError, msvcrt.get_osfhandle, make_bad_fd())

    def testBadModeArgument(self):
        bad_mode = 'qwerty'
        try:
            f = self.FileIO(TESTFN, bad_mode)
        except ValueError as msg:
            if msg.args[0] != 0:
                s = str(msg)
                if TESTFN in s or bad_mode not in s:
                    self.fail('bad error message for invalid mode: %s' % s)
        else:
            f.close()
            self.fail('no error for invalid mode: %s' % bad_mode)

    def testTruncate(self):
        f = self.FileIO(TESTFN, 'w')
        f.write(bytes(bytearray(range(10))))
        self.assertEqual(f.tell(), 10)
        f.truncate(5)
        self.assertEqual(f.tell(), 10)
        self.assertEqual(f.seek(0, io.SEEK_END), 5)
        f.truncate(15)
        self.assertEqual(f.tell(), 5)
        self.assertEqual(f.seek(0, io.SEEK_END), 15)
        f.close()

    def testTruncateOnWindows(self):

        def bug801631():
            f = self.FileIO(TESTFN, 'w')
            f.write(bytes(range(11)))
            f.close()
            f = self.FileIO(TESTFN, 'r+')
            data = f.read(5)
            if data != bytes(range(5)):
                self.fail('Read on file opened for update failed %r' % data)
            if f.tell() != 5:
                self.fail('File pos after read wrong %d' % f.tell())
            f.truncate()
            if f.tell() != 5:
                self.fail('File pos after ftruncate wrong %d' % f.tell())
            f.close()
            size = os.path.getsize(TESTFN)
            if size != 5:
                self.fail('File size after ftruncate wrong %d' % size)
        try:
            bug801631()
        finally:
            os.unlink(TESTFN)

    def testAppend(self):
        try:
            f = open(TESTFN, 'wb')
            f.write(b'spam')
            f.close()
            f = open(TESTFN, 'ab')
            f.write(b'eggs')
            f.close()
            f = open(TESTFN, 'rb')
            d = f.read()
            f.close()
            self.assertEqual(d, b'spameggs')
        finally:
            try:
                os.unlink(TESTFN)
            except:
                pass

    def testInvalidInit(self):
        self.assertRaises(TypeError, self.FileIO, '1', 0, 0)

    def testWarnings(self):
        with check_warnings(quiet=True) as w:
            self.assertEqual(w.warnings, [])
            self.assertRaises(TypeError, self.FileIO, [])
            self.assertEqual(w.warnings, [])
            self.assertRaises(ValueError, self.FileIO, '/some/invalid/name', 'rt')
            self.assertEqual(w.warnings, [])

    def testUnclosedFDOnException(self):

        class MyException(Exception):
            pass

        class MyFileIO(self.FileIO):

            def __setattr__(self, name, value):
                if name == 'name':
                    raise MyException('blocked setting name')
                return super(MyFileIO, self).__setattr__(name, value)
        fd = os.open(__file__, os.O_RDONLY)
        self.assertRaises(MyException, MyFileIO, fd)
        os.close(fd)

def tearDownModule():
    if os.path.exists(TESTFN):
        os.unlink(TESTFN)


# --- test body ---
FileIO = _io.FileIO
modulename = '_io'

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

def testAttributes():
    f = self_f

    assert f.mode == 'wb'

    assert f.closed == False
    for attr in ('mode', 'closed'):

        try:
            setattr(f, attr, 'oops')
            raise AssertionError('expected (AttributeError, TypeError)')
        except (AttributeError, TypeError):
            pass

def testBlksize():
    blksize = io.DEFAULT_BUFFER_SIZE
    if hasattr(os, 'fstat'):
        fst = os.fstat(self_f.fileno())
        blksize = getattr(fst, 'st_blksize', blksize)

    assert self_f._blksize == blksize

def testErrnoOnClose(f):
    f.close()

def testErrnoOnClosedFileno(f):
    f.fileno()

def testErrnoOnClosedIsatty(f):

    assert f.isatty() == False

def testErrnoOnClosedRead(f):
    f = ReopenForRead()
    f.read(1)

def testErrnoOnClosedReadable(f):
    f.readable()

def testErrnoOnClosedReadall(f):
    f = ReopenForRead()
    f.readall()

def testErrnoOnClosedReadinto(f):
    f = ReopenForRead()
    a = array('b', b'x' * 10)
    f.readinto(a)

def testErrnoOnClosedSeek(f):
    f.seek(0)

def testErrnoOnClosedSeekable(f):
    f.seekable()

def testErrnoOnClosedTell(f):
    f.tell()

def testErrnoOnClosedTruncate(f):
    f.truncate(0)

def testErrnoOnClosedWritable(f):
    f.writable()

def testErrnoOnClosedWrite(f):
    f.write(b'a')

def testErrors():
    f = self_f

    assert not f.isatty()

    assert not f.closed

    try:
        f.read(10)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    f.close()

    assert f.closed
    f = FileIO(TESTFN, 'r')

    try:
        f.readinto('')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    assert not f.closed
    f.close()

    assert f.closed

def testMethods():
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

def testOpenDirFD():
    fd = os.open('.', os.O_RDONLY)
    try:
        FileIO(fd, 'r')
        raise AssertionError('expected OSError')
    except OSError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    os.close(fd)

    assert cm.exception.errno == errno.EISDIR

def testOpendir():
    try:
        FileIO('.', 'r')
    except OSError as e:

        assert e.errno != 0

        assert e.filename == '.'
    else:

        raise AssertionError('Should have raised OSError')

def testReadintoByteArray():
    self_f.write(bytes([1, 2, 0, 255]))
    self_f.close()
    ba = bytearray(b'abcdefgh')
    with FileIO(TESTFN, 'r') as f:
        n = f.readinto(ba)

    assert ba == b'\x01\x02\x00\xffefgh'

    assert n == 4

def testRecursiveRepr():
    with swap_attr(self_f, 'name', self_f):
        try:
            repr(self_f)
            raise AssertionError('expected RuntimeError')
        except RuntimeError:
            pass

def testRepr():

    assert repr(self_f) == '<%s.FileIO name=%r mode=%r closefd=True>' % (modulename, self_f.name, self_f.mode)
    del self_f.name

    assert repr(self_f) == '<%s.FileIO fd=%r mode=%r closefd=True>' % (modulename, self_f.fileno(), self_f.mode)
    self_f.close()

    assert repr(self_f) == '<%s.FileIO [closed]>' % (modulename,)

def testReprNoCloseFD():
    fd = os.open(TESTFN, os.O_RDONLY)
    try:
        with FileIO(fd, 'r', closefd=False) as f:

            assert repr(f) == '<%s.FileIO name=%r mode=%r closefd=False>' % (modulename, f.name, f.mode)
    finally:
        os.close(fd)

def testSeekTell():
    self_f.write(bytes(range(20)))

    assert self_f.tell() == 20
    self_f.seek(0)

    assert self_f.tell() == 0
    self_f.seek(10)

    assert self_f.tell() == 10
    self_f.seek(5, 1)

    assert self_f.tell() == 15
    self_f.seek(-5, 1)

    assert self_f.tell() == 10
    self_f.seek(-5, 2)

    assert self_f.tell() == 15

def testWeakRefs():
    p = proxy(self_f)
    p.write(bytes(range(10)))

    assert self_f.tell() == p.tell()
    self_f.close()
    self_f = None
    gc_collect()

    try:
        getattr(p, 'tell')
        raise AssertionError('expected ReferenceError')
    except ReferenceError:
        pass

def testWritelinesError():

    try:
        self_f.writelines([1, 2, 3])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        self_f.writelines(None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        self_f.writelines('abc')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def testWritelinesList():
    l = [b'123', b'456']
    self_f.writelines(l)
    self_f.close()
    self_f = FileIO(TESTFN, 'rb')
    buf = self_f.read()

    assert buf == b'123456'

def testWritelinesUserList():
    l = UserList([b'123', b'456'])
    self_f.writelines(l)
    self_f.close()
    self_f = FileIO(TESTFN, 'rb')
    buf = self_f.read()

    assert buf == b'123456'
self_f = FileIO(TESTFN, 'w')
self_f.write(b'hi\nbye\nabc')
self_f.close()
self_f = FileIO(TESTFN, 'r')

assert self_f.read(None) == b'hi\nbye\nabc'
self_f.seek(0)

assert self_f.readline(None) == b'hi\n'

assert self_f.readlines(None) == [b'bye\n', b'abc']
print("CAutoFileTests::test_none_args: ok")
