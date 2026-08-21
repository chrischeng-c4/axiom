# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "test_redirect_stdout__test_cm_is_reusable"
# subject = "cpython.test_contextlib.TestRedirectStdout.test_cm_is_reusable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib.py::TestRedirectStdout::test_cm_is_reusable
"""Auto-ported test: TestRedirectStdout::test_cm_is_reusable (CPython 3.12 oracle)."""


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
redirect_stream = None
orig_stream = None
redirect_stream = redirect_stdout
orig_stream = 'stdout'
f = io.StringIO()
write_to_f = redirect_stream(f)
orig_stdout = getattr(sys, orig_stream)
with write_to_f:
    print('Hello', end=' ', file=getattr(sys, orig_stream))
with write_to_f:
    print('World!', file=getattr(sys, orig_stream))

assert getattr(sys, orig_stream) is orig_stdout
s = f.getvalue()

assert s == 'Hello World!\n'
print("TestRedirectStdout::test_cm_is_reusable: ok")
