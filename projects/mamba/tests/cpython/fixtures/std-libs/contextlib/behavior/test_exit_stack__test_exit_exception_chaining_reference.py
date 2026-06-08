# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "test_exit_stack__test_exit_exception_chaining_reference"
# subject = "cpython.test_contextlib.TestExitStack.test_exit_exception_chaining_reference"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib.py::TestExitStack::test_exit_exception_chaining_reference
"""Auto-ported test: TestExitStack::test_exit_exception_chaining_reference (CPython 3.12 oracle)."""


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

class RaiseExc:

    def __init__(self, exc):
        self.exc = exc

    def __enter__(self):
        return self

    def __exit__(self, *exc_details):
        raise self.exc

class RaiseExcWithContext:

    def __init__(self, outer, inner):
        self.outer = outer
        self.inner = inner

    def __enter__(self):
        return self

    def __exit__(self, *exc_details):
        try:
            raise self.inner
        except:
            raise self.outer

class SuppressExc:

    def __enter__(self):
        return self

    def __exit__(self, *exc_details):
        type(self).saved_details = exc_details
        return True
try:
    with RaiseExc(IndexError):
        with RaiseExcWithContext(KeyError, AttributeError):
            with SuppressExc():
                with RaiseExc(ValueError):
                    1 / 0
except IndexError as exc:

    assert isinstance(exc.__context__, KeyError)

    assert isinstance(exc.__context__.__context__, AttributeError)

    assert exc.__context__.__context__.__context__ is None
else:

    raise AssertionError('Expected IndexError, but no exception was raised')
inner_exc = SuppressExc.saved_details[1]

assert isinstance(inner_exc, ValueError)

assert isinstance(inner_exc.__context__, ZeroDivisionError)
print("TestExitStack::test_exit_exception_chaining_reference: ok")
