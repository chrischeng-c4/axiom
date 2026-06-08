# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "thread"
# dimension = "behavior"
# case = "thread_running_tests__test_count"
# subject = "cpython.test_thread.ThreadRunningTests.test__count"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_thread.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_thread.py::ThreadRunningTests::test__count
"""Auto-ported test: ThreadRunningTests::test__count (CPython 3.12 oracle)."""


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
orig = thread._count()
mut = thread.allocate_lock()
mut.acquire()
started = []

def task():
    started.append(None)
    mut.acquire()
    mut.release()
with threading_helper.wait_threads_exit():
    thread.start_new_thread(task, ())
    for _ in support.sleeping_retry(support.LONG_TIMEOUT):
        if started:
            break

    assert thread._count() == orig + 1
    mut.release()
    done = []
    wr = weakref.ref(task, lambda _: done.append(None))
    del task
    for _ in support.sleeping_retry(support.LONG_TIMEOUT):
        if done:
            break
        support.gc_collect()

    assert thread._count() == orig
print("ThreadRunningTests::test__count: ok")
