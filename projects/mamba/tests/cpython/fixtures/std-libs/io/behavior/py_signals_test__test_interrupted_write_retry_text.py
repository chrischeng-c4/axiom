# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "py_signals_test__test_interrupted_write_retry_text"
# subject = "cpython.test_io.PySignalsTest.test_interrupted_write_retry_text"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_io.py::PySignalsTest::test_interrupted_write_retry_text
"""Auto-ported test: PySignalsTest::test_interrupted_write_retry_text (CPython 3.12 oracle)."""


import abc
import array
import errno
import locale
import os
import pickle
import random
import signal
import sys
import textwrap
import threading
import time
import unittest
import warnings
import weakref
from collections import deque, UserList
from itertools import cycle, count
from test import support
from test.support.script_helper import assert_python_ok, assert_python_failure, run_python_until_end
from test.support import import_helper
from test.support import os_helper
from test.support import threading_helper
from test.support import warnings_helper
from test.support import skip_if_sanitizer
from test.support.os_helper import FakePath
import codecs
import io
import _pyio as pyio


'Unit tests for the io module.'

try:
    import ctypes
except ImportError:

    def byteslike(*pos, **kw):
        return array.array('b', bytes(*pos, **kw))
else:

    def byteslike(*pos, **kw):
        """Create a bytes-like object having no string or sequence methods"""
        data = bytes(*pos, **kw)
        obj = EmptyStruct()
        ctypes.resize(obj, len(data))
        memoryview(obj).cast('B')[:] = data
        return obj

    class EmptyStruct(ctypes.Structure):
        pass

IOBASE_EMITS_UNRAISABLE = support.Py_DEBUG or sys.flags.dev_mode

def _default_chunk_size():
    """Get the default TextIOWrapper chunk size"""
    with open(__file__, 'r', encoding='latin-1') as f:
        return f._CHUNK_SIZE

requires_alarm = unittest.skipUnless(hasattr(signal, 'alarm'), 'test requires signal.alarm()')

class BadIndex:

    def __index__(self):
        1 / 0

class MockRawIOWithoutRead:
    """A RawIO implementation without read(), so as to exercise the default
    RawIO.read() which calls readinto()."""

    def __init__(self, read_stack=()):
        self._read_stack = list(read_stack)
        self._write_stack = []
        self._reads = 0
        self._extraneous_reads = 0

    def write(self, b):
        self._write_stack.append(bytes(b))
        return len(b)

    def writable(self):
        return True

    def fileno(self):
        return 42

    def readable(self):
        return True

    def seekable(self):
        return True

    def seek(self, pos, whence):
        return 0

    def tell(self):
        return 0

    def readinto(self, buf):
        self._reads += 1
        max_len = len(buf)
        try:
            data = self._read_stack[0]
        except IndexError:
            self._extraneous_reads += 1
            return 0
        if data is None:
            del self._read_stack[0]
            return None
        n = len(data)
        if len(data) <= max_len:
            del self._read_stack[0]
            buf[:n] = data
            return n
        else:
            buf[:] = data[:max_len]
            self._read_stack[0] = data[max_len:]
            return max_len

    def truncate(self, pos=None):
        return pos

class CMockRawIOWithoutRead(MockRawIOWithoutRead, io.RawIOBase):
    pass

class PyMockRawIOWithoutRead(MockRawIOWithoutRead, pyio.RawIOBase):
    pass

class MockRawIO(MockRawIOWithoutRead):

    def read(self, n=None):
        self._reads += 1
        try:
            return self._read_stack.pop(0)
        except:
            self._extraneous_reads += 1
            return b''

class CMockRawIO(MockRawIO, io.RawIOBase):
    pass

class PyMockRawIO(MockRawIO, pyio.RawIOBase):
    pass

class MisbehavedRawIO(MockRawIO):

    def write(self, b):
        return super().write(b) * 2

    def read(self, n=None):
        return super().read(n) * 2

    def seek(self, pos, whence):
        return -123

    def tell(self):
        return -456

    def readinto(self, buf):
        super().readinto(buf)
        return len(buf) * 5

class CMisbehavedRawIO(MisbehavedRawIO, io.RawIOBase):
    pass

class PyMisbehavedRawIO(MisbehavedRawIO, pyio.RawIOBase):
    pass

class SlowFlushRawIO(MockRawIO):

    def __init__(self):
        super().__init__()
        self.in_flush = threading.Event()

    def flush(self):
        self.in_flush.set()
        time.sleep(0.25)

class CSlowFlushRawIO(SlowFlushRawIO, io.RawIOBase):
    pass

class PySlowFlushRawIO(SlowFlushRawIO, pyio.RawIOBase):
    pass

class CloseFailureIO(MockRawIO):
    closed = 0

    def close(self):
        if not self.closed:
            self.closed = 1
            raise OSError

class CCloseFailureIO(CloseFailureIO, io.RawIOBase):
    pass

class PyCloseFailureIO(CloseFailureIO, pyio.RawIOBase):
    pass

class MockFileIO:

    def __init__(self, data):
        self.read_history = []
        super().__init__(data)

    def read(self, n=None):
        res = super().read(n)
        self.read_history.append(None if res is None else len(res))
        return res

    def readinto(self, b):
        res = super().readinto(b)
        self.read_history.append(res)
        return res

class CMockFileIO(MockFileIO, io.BytesIO):
    pass

class PyMockFileIO(MockFileIO, pyio.BytesIO):
    pass

class MockUnseekableIO:

    def seekable(self):
        return False

    def seek(self, *args):
        raise self.UnsupportedOperation('not seekable')

    def tell(self, *args):
        raise self.UnsupportedOperation('not seekable')

    def truncate(self, *args):
        raise self.UnsupportedOperation('not seekable')

class CMockUnseekableIO(MockUnseekableIO, io.BytesIO):
    UnsupportedOperation = io.UnsupportedOperation

class PyMockUnseekableIO(MockUnseekableIO, pyio.BytesIO):
    UnsupportedOperation = pyio.UnsupportedOperation

class MockCharPseudoDevFileIO(MockFileIO):

    def __init__(self, data):
        super().__init__(data)

    def seek(self, *args):
        return 0

    def tell(self, *args):
        return 0

class CMockCharPseudoDevFileIO(MockCharPseudoDevFileIO, io.BytesIO):
    pass

class PyMockCharPseudoDevFileIO(MockCharPseudoDevFileIO, pyio.BytesIO):
    pass

class MockNonBlockWriterIO:

    def __init__(self):
        self._write_stack = []
        self._blocker_char = None

    def pop_written(self):
        s = b''.join(self._write_stack)
        self._write_stack[:] = []
        return s

    def block_on(self, char):
        """Block when a given char is encountered."""
        self._blocker_char = char

    def readable(self):
        return True

    def seekable(self):
        return True

    def seek(self, pos, whence=0):
        return 0

    def writable(self):
        return True

    def write(self, b):
        b = bytes(b)
        n = -1
        if self._blocker_char:
            try:
                n = b.index(self._blocker_char)
            except ValueError:
                pass
            else:
                if n > 0:
                    self._write_stack.append(b[:n])
                    return n
                else:
                    self._blocker_char = None
                    return None
        self._write_stack.append(b)
        return len(b)

class CMockNonBlockWriterIO(MockNonBlockWriterIO, io.RawIOBase):
    BlockingIOError = io.BlockingIOError

class PyMockNonBlockWriterIO(MockNonBlockWriterIO, pyio.RawIOBase):
    BlockingIOError = pyio.BlockingIOError

class StatefulIncrementalDecoder(codecs.IncrementalDecoder):
    """
    For testing seek/tell behavior with a stateful, buffering decoder.

    Input is a sequence of words.  Words may be fixed-length (length set
    by input) or variable-length (period-terminated).  In variable-length
    mode, extra periods are ignored.  Possible words are:
      - 'i' followed by a number sets the input length, I (maximum 99).
        When I is set to 0, words are space-terminated.
      - 'o' followed by a number sets the output length, O (maximum 99).
      - Any other word is converted into a word followed by a period on
        the output.  The output word consists of the input word truncated
        or padded out with hyphens to make its length equal to O.  If O
        is 0, the word is output verbatim without truncating or padding.
    I and O are initially set to 1.  When I changes, any buffered input is
    re-scanned according to the new I.  EOF also terminates the last word.
    """

    def __init__(self, errors='strict'):
        codecs.IncrementalDecoder.__init__(self, errors)
        self.reset()

    def __repr__(self):
        return '<SID %x>' % id(self)

    def reset(self):
        self.i = 1
        self.o = 1
        self.buffer = bytearray()

    def getstate(self):
        i, o = (self.i ^ 1, self.o ^ 1)
        return (bytes(self.buffer), i * 100 + o)

    def setstate(self, state):
        buffer, io = state
        self.buffer = bytearray(buffer)
        i, o = divmod(io, 100)
        self.i, self.o = (i ^ 1, o ^ 1)

    def decode(self, input, final=False):
        output = ''
        for b in input:
            if self.i == 0:
                if b == ord('.'):
                    if self.buffer:
                        output += self.process_word()
                else:
                    self.buffer.append(b)
            else:
                self.buffer.append(b)
                if len(self.buffer) == self.i:
                    output += self.process_word()
        if final and self.buffer:
            output += self.process_word()
        return output

    def process_word(self):
        output = ''
        if self.buffer[0] == ord('i'):
            self.i = min(99, int(self.buffer[1:] or 0))
        elif self.buffer[0] == ord('o'):
            self.o = min(99, int(self.buffer[1:] or 0))
        else:
            output = self.buffer.decode('ascii')
            if len(output) < self.o:
                output += '-' * self.o
            if self.o:
                output = output[:self.o]
            output += '.'
        self.buffer = bytearray()
        return output
    codecEnabled = False

def lookupTestDecoder(name):
    if StatefulIncrementalDecoder.codecEnabled and name == 'test_decoder':
        latin1 = codecs.lookup('latin-1')
        return codecs.CodecInfo(name='test_decoder', encode=latin1.encode, decode=None, incrementalencoder=None, streamreader=None, streamwriter=None, incrementaldecoder=StatefulIncrementalDecoder)

class MemviewBytesIO(io.BytesIO):
    """A BytesIO object whose read method returns memoryviews
       rather than bytes"""

    def read1(self, len_):
        return _to_memoryview(super().read1(len_))

    def read(self, len_):
        return _to_memoryview(super().read(len_))

def _to_memoryview(buf):
    """Convert bytes-object *buf* to a non-trivial memoryview"""
    arr = array.array('i')
    idx = len(buf) - len(buf) % arr.itemsize
    arr.frombytes(buf[:idx])
    return memoryview(arr)

def load_tests(loader, tests, pattern):
    tests = (CIOTest, PyIOTest, APIMismatchTest, CBufferedReaderTest, PyBufferedReaderTest, CBufferedWriterTest, PyBufferedWriterTest, CBufferedRWPairTest, PyBufferedRWPairTest, CBufferedRandomTest, PyBufferedRandomTest, StatefulIncrementalDecoderTest, CIncrementalNewlineDecoderTest, PyIncrementalNewlineDecoderTest, CTextIOWrapperTest, PyTextIOWrapperTest, CMiscIOTest, PyMiscIOTest, CSignalsTest, PySignalsTest, TestIOCTypes)
    mocks = (MockRawIO, MisbehavedRawIO, MockFileIO, CloseFailureIO, MockNonBlockWriterIO, MockUnseekableIO, MockRawIOWithoutRead, SlowFlushRawIO, MockCharPseudoDevFileIO)
    all_members = io.__all__
    c_io_ns = {name: getattr(io, name) for name in all_members}
    py_io_ns = {name: getattr(pyio, name) for name in all_members}
    globs = globals()
    c_io_ns.update(((x.__name__, globs['C' + x.__name__]) for x in mocks))
    py_io_ns.update(((x.__name__, globs['Py' + x.__name__]) for x in mocks))
    for test in tests:
        if test.__name__.startswith('C'):
            for name, obj in c_io_ns.items():
                setattr(test, name, obj)
            test.is_C = True
        elif test.__name__.startswith('Py'):
            for name, obj in py_io_ns.items():
                setattr(test, name, obj)
            test.is_C = False
    suite = loader.suiteClass()
    for test in tests:
        suite.addTest(loader.loadTestsFromTestCase(test))
    return suite


# --- test body ---
io = pyio
test_reentrant_write_buffered = None
test_reentrant_write_text = None

def alarm_interrupt(sig, frame):
    1 / 0

def check_interrupted_read_retry(decode, **fdopen_kwargs):
    """Check that a buffered read, when it gets interrupted (either
        returning a partial result or EINTR), properly invokes the signal
        handler and retries if the latter returned successfully."""
    r, w = os.pipe()
    fdopen_kwargs['closefd'] = False

    def alarm_handler(sig, frame):
        os.write(w, b'bar')
    signal.signal(signal.SIGALRM, alarm_handler)
    try:
        rio = io.open(r, **fdopen_kwargs)
        os.write(w, b'foo')
        signal.alarm(1)

        assert decode(rio.read(6)) == 'foobar'
    finally:
        signal.alarm(0)
        rio.close()
        os.close(w)
        os.close(r)

def check_interrupted_write(item, bytes, **fdopen_kwargs):
    """Check that a partial write, when it gets interrupted, properly
        invokes the signal handler, and bubbles up the exception raised
        in the latter."""
    support.gc_collect()
    read_results = []

    def _read():
        s = os.read(r, 1)
        read_results.append(s)
    t = threading.Thread(target=_read)
    t.daemon = True
    r, w = os.pipe()
    fdopen_kwargs['closefd'] = False
    large_data = item * (support.PIPE_MAX_SIZE // len(item) + 1)
    try:
        wio = io.open(w, **fdopen_kwargs)
        if hasattr(signal, 'pthread_sigmask'):
            signal.pthread_sigmask(signal.SIG_BLOCK, [signal.SIGALRM])
            t.start()
            signal.pthread_sigmask(signal.SIG_UNBLOCK, [signal.SIGALRM])
        else:
            t.start()
        signal.alarm(1)
        try:

            try:
                wio.write(large_data)
                raise AssertionError('expected ZeroDivisionError')
            except ZeroDivisionError:
                pass
        finally:
            signal.alarm(0)
            t.join()
        read_results.append(os.read(r, 1))

        assert read_results == [bytes[0:1], bytes[1:2]]
    finally:
        os.close(w)
        os.close(r)
        try:
            wio.close()
        except OSError as e:
            if e.errno != errno.EBADF:
                raise

def check_interrupted_write_retry(item, **fdopen_kwargs):
    """Check that a buffered write, when it gets interrupted (either
        returning a partial result or EINTR), properly invokes the signal
        handler and retries if the latter returned successfully."""
    select = import_helper.import_module('select')
    N = support.PIPE_MAX_SIZE
    r, w = os.pipe()
    fdopen_kwargs['closefd'] = False
    read_results = []
    write_finished = False
    error = None

    def _read():
        try:
            while not write_finished:
                while r in select.select([r], [], [], 1.0)[0]:
                    s = os.read(r, 1024)
                    read_results.append(s)
        except BaseException as exc:
            nonlocal error
            error = exc
    t = threading.Thread(target=_read)
    t.daemon = True

    def alarm1(sig, frame):
        signal.signal(signal.SIGALRM, alarm2)
        signal.alarm(1)

    def alarm2(sig, frame):
        t.start()
    large_data = item * N
    signal.signal(signal.SIGALRM, alarm1)
    try:
        wio = io.open(w, **fdopen_kwargs)
        signal.alarm(1)
        written = wio.write(large_data)

        assert N == written
        wio.flush()
        write_finished = True
        t.join()

        assert error is None

        assert N == sum((len(x) for x in read_results))
    finally:
        signal.alarm(0)
        write_finished = True
        os.close(w)
        os.close(r)
        try:
            wio.close()
        except OSError as e:
            if e.errno != errno.EBADF:
                raise

def check_reentrant_write(data, **fdopen_kwargs):

    def on_alarm(*args):
        wio.write(data)
        1 / 0
    signal.signal(signal.SIGALRM, on_alarm)
    r, w = os.pipe()
    wio = io.open(w, **fdopen_kwargs)
    try:
        signal.alarm(1)
        try:
            while 1:
                for i in range(100):
                    wio.write(data)
                    wio.flush()
                os.read(r, len(data) * 100)
            raise AssertionError('expected (ZeroDivisionError, RuntimeError)')
        except (ZeroDivisionError, RuntimeError) as _aR_e:
            import types as _types_aR
            cm = _types_aR.SimpleNamespace(exception=_aR_e)
        exc = cm.exception
        if isinstance(exc, RuntimeError):

            assert str(exc).startswith('reentrant call')
    finally:
        signal.alarm(0)
        wio.close()
        os.close(r)
self_oldalrm = signal.signal(signal.SIGALRM, alarm_interrupt)
check_interrupted_write_retry('x', mode='w', encoding='latin1')
print("PySignalsTest::test_interrupted_write_retry_text: ok")
