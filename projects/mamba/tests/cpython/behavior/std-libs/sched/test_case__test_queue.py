# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "behavior"
# case = "test_case__test_queue"
# subject = "cpython.test_sched.TestCase.test_queue"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sched.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sched.py::TestCase::test_queue
"""Auto-ported test: TestCase::test_queue (CPython 3.12 oracle)."""


import queue
import sched
import threading
import time
import unittest
from test import support
from test.support import threading_helper


TIMEOUT = support.SHORT_TIMEOUT

class Timer:

    def __init__(self):
        self._cond = threading.Condition()
        self._time = 0
        self._stop = 0

    def time(self):
        with self._cond:
            return self._time

    def sleep(self, t):
        assert t >= 0
        with self._cond:
            t += self._time
            while self._stop < t:
                self._time = self._stop
                self._cond.wait()
            self._time = t

    def advance(self, t):
        assert t >= 0
        with self._cond:
            self._stop += t
            self._cond.notify_all()


# --- test body ---
l = []
fun = lambda x: l.append(x)
scheduler = sched.scheduler(time.time, time.sleep)
now = time.time()
e5 = scheduler.enterabs(now + 0.05, 1, fun)
e1 = scheduler.enterabs(now + 0.01, 1, fun)
e2 = scheduler.enterabs(now + 0.02, 1, fun)
e4 = scheduler.enterabs(now + 0.04, 1, fun)
e3 = scheduler.enterabs(now + 0.03, 1, fun)

assert scheduler.queue == [e1, e2, e3, e4, e5]
print("TestCase::test_queue: ok")
