# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poll"
# dimension = "behavior"
# case = "poll_tests__test_poll1"
# subject = "cpython.test_poll.PollTests.test_poll1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_poll.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_poll.py::PollTests::test_poll1
"""Auto-ported test: PollTests::test_poll1 (CPython 3.12 oracle)."""


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
p = select.poll()
NUM_PIPES = 12
MSG = b' This is a test.'
MSG_LEN = len(MSG)
readers = []
writers = []
r2w = {}
w2r = {}
for i in range(NUM_PIPES):
    rd, wr = os.pipe()
    p.register(rd)
    p.modify(rd, select.POLLIN)
    p.register(wr, select.POLLOUT)
    readers.append(rd)
    writers.append(wr)
    r2w[rd] = wr
    w2r[wr] = rd
bufs = []
while writers:
    ready = p.poll()
    ready_writers = find_ready_matching(ready, select.POLLOUT)
    if not ready_writers:
        raise RuntimeError('no pipes ready for writing')
    wr = random.choice(ready_writers)
    os.write(wr, MSG)
    ready = p.poll()
    ready_readers = find_ready_matching(ready, select.POLLIN)
    if not ready_readers:
        raise RuntimeError('no pipes ready for reading')
    rd = random.choice(ready_readers)
    buf = os.read(rd, MSG_LEN)

    assert len(buf) == MSG_LEN
    bufs.append(buf)
    os.close(r2w[rd])
    os.close(rd)
    p.unregister(r2w[rd])
    p.unregister(rd)
    writers.remove(r2w[rd])

assert bufs == [MSG] * NUM_PIPES
print("PollTests::test_poll1: ok")
