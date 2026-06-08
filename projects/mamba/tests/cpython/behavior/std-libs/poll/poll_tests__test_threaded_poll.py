# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poll"
# dimension = "behavior"
# case = "poll_tests__test_threaded_poll"
# subject = "cpython.test_poll.PollTests.test_threaded_poll"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_poll.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_poll.py::PollTests::test_threaded_poll
"""Auto-ported test: PollTests::test_threaded_poll (CPython 3.12 oracle)."""


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
r, w = os.pipe()
pass
pass
rfds = []
for i in range(10):
    fd = os.dup(r)
    pass
    rfds.append(fd)
pollster = select.poll()
for fd in rfds:
    pollster.register(fd, select.POLLIN)
t = threading.Thread(target=pollster.poll)
t.start()
try:
    time.sleep(0.5)
    for fd in rfds:
        pollster.unregister(fd)
    pollster.register(w, select.POLLOUT)

    try:
        pollster.poll()
        raise AssertionError('expected RuntimeError')
    except RuntimeError:
        pass
finally:
    os.write(w, b'spam')
    t.join()
print("PollTests::test_threaded_poll: ok")
