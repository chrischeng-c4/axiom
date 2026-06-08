# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "behavior"
# case = "test_case__test_enter_concurrent"
# subject = "cpython.test_sched.TestCase.test_enter_concurrent"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sched.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sched.py::TestCase::test_enter_concurrent
"""Auto-ported test: TestCase::test_enter_concurrent (CPython 3.12 oracle)."""


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
q = queue.Queue()
fun = q.put
timer = Timer()
scheduler = sched.scheduler(timer.time, timer.sleep)
scheduler.enter(1, 1, fun, (1,))
scheduler.enter(3, 1, fun, (3,))
t = threading.Thread(target=scheduler.run)
t.start()
timer.advance(1)

assert q.get(timeout=TIMEOUT) == 1

assert q.empty()
for x in [4, 5, 2]:
    z = scheduler.enter(x - 1, 1, fun, (x,))
timer.advance(2)

assert q.get(timeout=TIMEOUT) == 2

assert q.get(timeout=TIMEOUT) == 3

assert q.empty()
timer.advance(1)

assert q.get(timeout=TIMEOUT) == 4

assert q.empty()
timer.advance(1)

assert q.get(timeout=TIMEOUT) == 5

assert q.empty()
timer.advance(1000)
threading_helper.join_thread(t)

assert q.empty()

assert timer.time() == 5
print("TestCase::test_enter_concurrent: ok")
