# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "test_exit_stack__test_callback"
# subject = "cpython.test_contextlib.TestExitStack.test_callback"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib.py::TestExitStack::test_callback
"""Auto-ported test: TestExitStack::test_callback (CPython 3.12 oracle)."""


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
expected = [((), {}), ((1,), {}), ((1, 2), {}), ((), dict(example=1)), ((1,), dict(example=1)), ((1, 2), dict(example=1)), ((1, 2), dict(self=3, callback=4))]
result = []

def _exit(*args, **kwds):
    """Test metadata propagation"""
    result.append((args, kwds))
with exit_stack() as stack:
    for args, kwds in reversed(expected):
        if args and kwds:
            f = stack.callback(_exit, *args, **kwds)
        elif args:
            f = stack.callback(_exit, *args)
        elif kwds:
            f = stack.callback(_exit, **kwds)
        else:
            f = stack.callback(_exit)

        assert f is _exit
    for wrapper in stack._exit_callbacks:

        assert wrapper[1].__wrapped__ is _exit

        assert wrapper[1].__name__ != _exit.__name__

        assert wrapper[1].__doc__ is None

assert result == expected
result = []
with exit_stack() as stack:
    try:
        stack.callback(arg=1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        exit_stack.callback(arg=2)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        stack.callback(callback=_exit, arg=3)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

assert result == []
print("TestExitStack::test_callback: ok")
