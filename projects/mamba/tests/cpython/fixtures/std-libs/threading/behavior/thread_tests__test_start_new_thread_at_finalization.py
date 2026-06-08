# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "thread_tests__test_start_new_thread_at_finalization"
# subject = "cpython.test_threading.ThreadTests.test_start_new_thread_at_finalization"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threading.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_threading.py::ThreadTests::test_start_new_thread_at_finalization
"""Auto-ported test: ThreadTests::test_start_new_thread_at_finalization (CPython 3.12 oracle)."""


import test.support
from test.support import threading_helper, requires_subprocess
from test.support import verbose, cpython_only, os_helper
from test.support.import_helper import import_module
from test.support.script_helper import assert_python_ok, assert_python_failure
import random
import sys
import _thread
import threading
import time
import unittest
import weakref
import os
import subprocess
import signal
import textwrap
import traceback
import warnings
from unittest import mock
from test import lock_tests
from test import support


'\nTests for the threading module.\n'

try:
    from test.support import interpreters
except ModuleNotFoundError:
    interpreters = None

threading_helper.requires_working_threading(module=True)

platforms_to_skip = ('netbsd5', 'hp-ux11')

def skip_unless_reliable_fork(test):
    if not support.has_fork_support:
        return unittest.skip('requires working os.fork()')(test)
    if sys.platform in platforms_to_skip:
        return unittest.skip('due to known OS bug related to thread+fork')(test)
    if support.HAVE_ASAN_FORK_BUG:
        return unittest.skip('libasan has a pthread_create() dead lock related to thread+fork')(test)
    if support.check_sanitizer(thread=True):
        return unittest.skip("TSAN doesn't support threads after fork")
    return test

def requires_subinterpreters(meth):
    """Decorator to skip a test if subinterpreters are not supported."""
    return unittest.skipIf(interpreters is None, 'subinterpreters required')(meth)

def restore_default_excepthook(testcase):
    testcase.addCleanup(setattr, threading, 'excepthook', threading.excepthook)
    threading.excepthook = threading.__excepthook__

class Counter(object):

    def __init__(self):
        self.value = 0

    def inc(self):
        self.value += 1

    def dec(self):
        self.value -= 1

    def get(self):
        return self.value

class BaseTestCase(unittest.TestCase):

    def setUp(self):
        self._threads = threading_helper.threading_setup()

    def tearDown(self):
        threading_helper.threading_cleanup(*self._threads)
        test.support.reap_children()

class ThreadRunFail(threading.Thread):

    def run(self):
        raise ValueError('run failed')

class LockTests(lock_tests.LockTests):
    locktype = staticmethod(threading.Lock)

class PyRLockTests(lock_tests.RLockTests):
    locktype = staticmethod(threading._PyRLock)

@unittest.skipIf(threading._CRLock is None, 'RLock not implemented in C')
class CRLockTests(lock_tests.RLockTests):
    locktype = staticmethod(threading._CRLock)

class EventTests(lock_tests.EventTests):
    eventtype = staticmethod(threading.Event)

class ConditionTests(lock_tests.ConditionTests):
    condtype = staticmethod(threading.Condition)

class SemaphoreTests(lock_tests.SemaphoreTests):
    semtype = staticmethod(threading.Semaphore)

class BoundedSemaphoreTests(lock_tests.BoundedSemaphoreTests):
    semtype = staticmethod(threading.BoundedSemaphore)

class BarrierTests(lock_tests.BarrierTests):
    barriertype = staticmethod(threading.Barrier)


# --- test body ---
maxDiff = 9999
self__threads = threading_helper.threading_setup()
code = 'if 1:\n            import _thread\n\n            def f():\n                print("shouldn\'t be printed")\n\n            class AtFinalization:\n                def __del__(self):\n                    print("OK")\n                    _thread.start_new_thread(f, ())\n            at_finalization = AtFinalization()\n        '
_, out, err = assert_python_ok('-c', code)

assert out.strip() == b'OK'

assert b"can't create new thread at interpreter shutdown" in err
print("ThreadTests::test_start_new_thread_at_finalization: ok")
