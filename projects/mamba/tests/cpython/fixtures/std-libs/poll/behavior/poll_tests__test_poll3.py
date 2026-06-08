# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poll"
# dimension = "behavior"
# case = "poll_tests__test_poll3"
# subject = "cpython.test_poll.PollTests.test_poll3"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_poll.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_poll.py::PollTests::test_poll3
"""Auto-ported test: PollTests::test_poll3 (CPython 3.12 oracle)."""


import os
import subprocess
import random
import select
import threading
import time
import unittest
from test.support import cpython_only, requires_subprocess, requires_working_socket, requires_resource
from test.support import threading_helper
from test.support.os_helper import TESTFN


try:
    select.poll
except AttributeError:
    raise unittest.SkipTest('select.poll not defined')

requires_working_socket(module=True)

def find_ready_matching(ready, flag):
    match = []
    for fd, mode in ready:
        if mode & flag:
            match.append(fd)
    return match


# --- test body ---
pollster = select.poll()
pollster.register(1)

try:
    pollster.poll(1 << 64)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass
x = 2 + 3
if x != 5:

    raise AssertionError('Overflow must have occurred')

try:
    pollster.register(0, -1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    pollster.register(0, 1 << 64)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    pollster.modify(1, -1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    pollster.modify(1, 1 << 64)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass
print("PollTests::test_poll3: ok")
