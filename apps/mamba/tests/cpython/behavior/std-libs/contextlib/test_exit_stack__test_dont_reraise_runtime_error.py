# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "test_exit_stack__test_dont_reraise_runtime_error"
# subject = "cpython.test_contextlib.TestExitStack.test_dont_reraise_RuntimeError"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib.py::TestExitStack::test_dont_reraise_RuntimeError
"""Auto-ported test: TestExitStack::test_dont_reraise_RuntimeError (CPython 3.12 oracle)."""


import io
import os
import sys
import tempfile
import threading
import traceback
import unittest
from contextlib import *
from test import support
from test.support import os_helper
from test.support.testcase import ExceptionIsLikeMixin
import weakref


'Unit tests for contextlib.py, and other context managers.'

class FileContextTestCase(unittest.TestCase):

    def testWithOpen(self):
        tfn = tempfile.mktemp()
        try:
            with open(tfn, 'w', encoding='utf-8') as f:
                self.assertFalse(f.closed)
                f.write('Booh\n')
            self.assertTrue(f.closed)
            with self.assertRaises(ZeroDivisionError):
                with open(tfn, 'r', encoding='utf-8') as f:
                    self.assertFalse(f.closed)
                    self.assertEqual(f.read(), 'Booh\n')
                    1 / 0
            self.assertTrue(f.closed)
        finally:
            os_helper.unlink(tfn)

class LockContextTestCase(unittest.TestCase):

    def boilerPlate(self, lock, locked):
        self.assertFalse(locked())
        with lock:
            self.assertTrue(locked())
        self.assertFalse(locked())
        with self.assertRaises(ZeroDivisionError):
            with lock:
                self.assertTrue(locked())
                1 / 0
        self.assertFalse(locked())

    def testWithLock(self):
        lock = threading.Lock()
        self.boilerPlate(lock, lock.locked)

    def testWithRLock(self):
        lock = threading.RLock()
        self.boilerPlate(lock, lock._is_owned)

    def testWithCondition(self):
        lock = threading.Condition()

        def locked():
            return lock._is_owned()
        self.boilerPlate(lock, locked)

    def testWithSemaphore(self):
        lock = threading.Semaphore()

        def locked():
            if lock.acquire(False):
                lock.release()
                return False
            else:
                return True
        self.boilerPlate(lock, locked)

    def testWithBoundedSemaphore(self):
        lock = threading.BoundedSemaphore()

        def locked():
            if lock.acquire(False):
                lock.release()
                return False
            else:
                return True
        self.boilerPlate(lock, locked)

class mycontext(ContextDecorator):
    """Example decoration-compatible context manager for testing"""
    started = False
    exc = None
    catch = False

    def __enter__(self):
        self.started = True
        return self

    def __exit__(self, *exc):
        self.exc = exc
        return self.catch


# --- test body ---
exit_stack = None
exit_stack = ExitStack
callback_error_internal_frames = [('__exit__', 'raise exc_details[1]'), ('__exit__', 'if cb(*exc_details):')]

class UniqueException(Exception):
    pass

class UniqueRuntimeError(RuntimeError):
    pass

@contextmanager
def second():
    try:
        yield 1
    except Exception as exc:
        raise UniqueException('new exception') from exc

@contextmanager
def first():
    try:
        yield 1
    except Exception as exc:
        raise exc
try:
    with exit_stack() as es_ctx:
        es_ctx.enter_context(second())
        es_ctx.enter_context(first())
        raise UniqueRuntimeError('please no infinite loop.')
    raise AssertionError('expected UniqueException')
except UniqueException as _aR_e:
    import types as _types_aR
    err_ctx = _types_aR.SimpleNamespace(exception=_aR_e)
exc = err_ctx.exception

assert isinstance(exc, UniqueException)

assert isinstance(exc.__context__, UniqueRuntimeError)

assert exc.__context__.__context__ is None

assert exc.__context__.__cause__ is None

assert exc.__cause__ is exc.__context__
print("TestExitStack::test_dont_reraise_RuntimeError: ok")
