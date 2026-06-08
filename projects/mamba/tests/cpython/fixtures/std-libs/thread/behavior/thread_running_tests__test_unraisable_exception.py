# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "thread"
# dimension = "behavior"
# case = "thread_running_tests__test_unraisable_exception"
# subject = "cpython.test_thread.ThreadRunningTests.test_unraisable_exception"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_thread.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_thread.py::ThreadRunningTests::test_unraisable_exception
"""Auto-ported test: ThreadRunningTests::test_unraisable_exception (CPython 3.12 oracle)."""


import os
import unittest
import random
from test import support
from test.support import threading_helper
import _thread as thread
import time
import warnings
import weakref
from test import lock_tests


threading_helper.requires_working_threading(module=True)

NUMTASKS = 10

NUMTRIPS = 3

_print_mutex = thread.allocate_lock()

def verbose_print(arg):
    """Helper function for printing out debugging output."""
    if support.verbose:
        with _print_mutex:
            print(arg)

class Barrier:

    def __init__(self, num_threads):
        self.num_threads = num_threads
        self.waiting = 0
        self.checkin_mutex = thread.allocate_lock()
        self.checkout_mutex = thread.allocate_lock()
        self.checkout_mutex.acquire()

    def enter(self):
        self.checkin_mutex.acquire()
        self.waiting = self.waiting + 1
        if self.waiting == self.num_threads:
            self.waiting = self.num_threads - 1
            self.checkout_mutex.release()
            return
        self.checkin_mutex.release()
        self.checkout_mutex.acquire()
        self.waiting = self.waiting - 1
        if self.waiting == 0:
            self.checkin_mutex.release()
            return
        self.checkout_mutex.release()

class LockTests(lock_tests.LockTests):
    locktype = thread.allocate_lock


# --- test body ---
def newtask():
    with self_running_mutex:
        self_next_ident += 1
        verbose_print('creating task %s' % self_next_ident)
        thread.start_new_thread(task, (self_next_ident,))
        self_created += 1
        self_running += 1

def task(ident):
    with self_random_mutex:
        delay = random.random() / 10000.0
    verbose_print('task %s will run for %sus' % (ident, round(delay * 1000000.0)))
    time.sleep(delay)
    verbose_print('task %s done' % ident)
    with self_running_mutex:
        self_running -= 1
        if self_created == NUMTASKS and self_running == 0:
            self_done_mutex.release()
self_done_mutex = thread.allocate_lock()
self_done_mutex.acquire()
self_running_mutex = thread.allocate_lock()
self_random_mutex = thread.allocate_lock()
self_created = 0
self_running = 0
self_next_ident = 0
key = threading_helper.threading_setup()
pass

def task():
    started.release()
    raise ValueError('task failed')
started = thread.allocate_lock()
with support.catch_unraisable_exception() as cm:
    with threading_helper.wait_threads_exit():
        started.acquire()
        thread.start_new_thread(task, ())
        started.acquire()

    assert str(cm.unraisable.exc_value) == 'task failed'

    assert cm.unraisable.object is task

    assert cm.unraisable.err_msg == 'Exception ignored in thread started by'

    assert cm.unraisable.exc_traceback is not None
print("ThreadRunningTests::test_unraisable_exception: ok")
