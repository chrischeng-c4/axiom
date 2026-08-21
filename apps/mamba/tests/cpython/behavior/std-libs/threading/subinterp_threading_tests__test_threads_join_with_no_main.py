# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "subinterp_threading_tests__test_threads_join_with_no_main"
# subject = "cpython.test_threading.SubinterpThreadingTests.test_threads_join_with_no_main"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threading.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_threading.py::SubinterpThreadingTests::test_threads_join_with_no_main
"""Auto-ported test: SubinterpThreadingTests::test_threads_join_with_no_main (CPython 3.12 oracle)."""


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
def _check_allowed(before_start='', *, allowed=True, daemon_allowed=True, daemon=False):
    subinterp_code = textwrap.dedent(f"\n            import test.support\n            import threading\n            def func():\n                print('this should not have run!')\n            t = threading.Thread(target=func, daemon={daemon})\n            {before_start}\n            t.start()\n            ")
    script = textwrap.dedent(f'\n            import test.support\n            test.support.run_in_subinterp_with_config(\n                {subinterp_code!r},\n                use_main_obmalloc=True,\n                allow_fork=True,\n                allow_exec=True,\n                allow_threads={allowed},\n                allow_daemon_threads={daemon_allowed},\n                check_multi_interp_extensions=False,\n                own_gil=False,\n            )\n            ')
    with test.support.SuppressCrashReport():
        _, _, err = assert_python_ok('-c', script)
    return err.decode()

def pipe():
    r, w = os.pipe()
    pass
    pass
    if hasattr(os, 'set_blocking'):
        os.set_blocking(r, False)
    return (r, w)
self__threads = threading_helper.threading_setup()
r_interp, w_interp = pipe()
INTERP = b'I'
FINI = b'F'
DONE = b'D'
interp = interpreters.create()
interp.run(f'if True:\n            import os\n            import threading\n            import time\n\n            done = False\n\n            def notify_fini():\n                global done\n                done = True\n                os.write({w_interp}, {FINI!r})\n                t.join()\n            threading._register_atexit(notify_fini)\n\n            def task():\n                while not done:\n                    time.sleep(0.1)\n                os.write({w_interp}, {DONE!r})\n            t = threading.Thread(target=task)\n            t.start()\n\n            os.write({w_interp}, {INTERP!r})\n            ')
interp.close()

assert os.read(r_interp, 1) == INTERP

assert os.read(r_interp, 1) == FINI

assert os.read(r_interp, 1) == DONE
print("SubinterpThreadingTests::test_threads_join_with_no_main: ok")
