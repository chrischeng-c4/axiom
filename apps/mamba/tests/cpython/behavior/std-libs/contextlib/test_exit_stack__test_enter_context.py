# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "test_exit_stack__test_enter_context"
# subject = "cpython.test_contextlib.TestExitStack.test_enter_context"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib.py::TestExitStack::test_enter_context
"""Auto-ported test: TestExitStack::test_enter_context (CPython 3.12 oracle)."""


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

class TestCM(object):

    def __enter__(self):
        result.append(1)

    def __exit__(self, *exc_details):
        result.append(3)
result = []
cm = TestCM()
with exit_stack() as stack:

    @stack.callback
    def _exit():
        result.append(4)

    assert _exit is not None
    stack.enter_context(cm)

    assert stack._exit_callbacks[-1][1].__self__ is cm
    result.append(2)

assert result == [1, 2, 3, 4]
print("TestExitStack::test_enter_context: ok")
