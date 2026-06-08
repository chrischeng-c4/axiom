# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poll"
# dimension = "behavior"
# case = "poll_tests__test_poll_c_limits"
# subject = "cpython.test_poll.PollTests.test_poll_c_limits"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_poll.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_poll.py::PollTests::test_poll_c_limits
"""Auto-ported test: PollTests::test_poll_c_limits (CPython 3.12 oracle)."""


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
from _testcapi import USHRT_MAX, INT_MAX, UINT_MAX
pollster = select.poll()
pollster.register(1)

try:
    pollster.register(0, USHRT_MAX + 1)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    pollster.modify(1, USHRT_MAX + 1)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    pollster.poll(INT_MAX + 1)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    pollster.poll(UINT_MAX + 1)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass
print("PollTests::test_poll_c_limits: ok")
