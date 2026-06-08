# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "behavior"
# case = "test_case__test_priority"
# subject = "cpython.test_sched.TestCase.test_priority"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sched.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sched.py::TestCase::test_priority
"""Auto-ported test: TestCase::test_priority (CPython 3.12 oracle)."""


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
cases = [([1, 2, 3, 4, 5], [1, 2, 3, 4, 5]), ([5, 4, 3, 2, 1], [1, 2, 3, 4, 5]), ([2, 5, 3, 1, 4], [1, 2, 3, 4, 5]), ([1, 2, 3, 2, 1], [1, 1, 2, 2, 3])]
for priorities, expected in cases:
    for priority in priorities:
        scheduler.enterabs(0.01, priority, fun, (priority,))
    scheduler.run()

    assert l == expected

    assert scheduler.empty()
    l.clear()
print("TestCase::test_priority: ok")
