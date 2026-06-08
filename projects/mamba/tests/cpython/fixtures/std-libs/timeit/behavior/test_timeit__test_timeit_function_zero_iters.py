# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "behavior"
# case = "test_timeit__test_timeit_function_zero_iters"
# subject = "cpython.test_timeit.TestTimeit.test_timeit_function_zero_iters"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_timeit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_timeit.py::TestTimeit::test_timeit_function_zero_iters
"""Auto-ported test: TestTimeit::test_timeit_function_zero_iters (CPython 3.12 oracle)."""


import timeit
import unittest
import sys
import io
from textwrap import dedent
from test.support import captured_stdout
from test.support import captured_stderr


DEFAULT_NUMBER = 1000000

DEFAULT_REPEAT = 5

class FakeTimer:
    BASE_TIME = 42.0

    def __init__(self, seconds_per_increment=1.0):
        self.count = 0
        self.setup_calls = 0
        self.seconds_per_increment = seconds_per_increment
        timeit._fake_timer = self

    def __call__(self):
        return self.BASE_TIME + self.count * self.seconds_per_increment

    def inc(self):
        self.count += 1

    def setup(self):
        self.setup_calls += 1

    def wrap_timer(self, timer):
        """Records 'timer' and returns self as callable timer."""
        self.saved_timer = timer
        return self


# --- test body ---
fake_setup = 'import timeit\ntimeit._fake_timer.setup()'
fake_stmt = 'import timeit\ntimeit._fake_timer.inc()'
MAIN_DEFAULT_OUTPUT = '1 loop, best of 5: 1 sec per loop\n'
delta_time = timeit.timeit(fake_stmt, fake_setup, number=0, timer=FakeTimer())

assert delta_time == 0
print("TestTimeit::test_timeit_function_zero_iters: ok")
