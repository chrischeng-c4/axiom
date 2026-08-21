# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poll"
# dimension = "behavior"
# case = "poll_tests__test_poll_unit_tests"
# subject = "cpython.test_poll.PollTests.test_poll_unit_tests"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_poll.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_poll.py::PollTests::test_poll_unit_tests
"""Auto-ported test: PollTests::test_poll_unit_tests (CPython 3.12 oracle)."""


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
FD, w = os.pipe()
os.close(FD)
os.close(w)
p = select.poll()
p.register(FD)
r = p.poll()

assert r[0] == (FD, select.POLLNVAL)
with open(TESTFN, 'w') as f:
    fd = f.fileno()
    p = select.poll()
    p.register(f)
    r = p.poll()

    assert r[0][0] == fd
r = p.poll()

assert r[0] == (fd, select.POLLNVAL)
os.unlink(TESTFN)
p = select.poll()

try:
    p.register(p)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    p.unregister(p)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
p = select.poll()

try:
    p.unregister(3)
    raise AssertionError('expected KeyError')
except KeyError:
    pass
pollster = select.poll()

class Nope:
    pass

class Almost:

    def fileno(self):
        return 'fileno'

try:
    pollster.register(Nope(), 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    pollster.register(Almost(), 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PollTests::test_poll_unit_tests: ok")
