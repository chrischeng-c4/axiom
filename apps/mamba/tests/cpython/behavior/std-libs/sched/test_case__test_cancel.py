# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "behavior"
# case = "test_case__test_cancel"
# subject = "cpython.test_sched.TestCase.test_cancel"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sched.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sched.py::TestCase::test_cancel
"""Auto-ported test: TestCase::test_cancel (CPython 3.12 oracle)."""


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
event1 = scheduler.enterabs(now + 0.01, 1, fun, (0.01,))
event2 = scheduler.enterabs(now + 0.02, 1, fun, (0.02,))
event3 = scheduler.enterabs(now + 0.03, 1, fun, (0.03,))
event4 = scheduler.enterabs(now + 0.04, 1, fun, (0.04,))
event5 = scheduler.enterabs(now + 0.05, 1, fun, (0.05,))
scheduler.cancel(event1)
scheduler.cancel(event5)
scheduler.run()

assert l == [0.02, 0.03, 0.04]
print("TestCase::test_cancel: ok")
