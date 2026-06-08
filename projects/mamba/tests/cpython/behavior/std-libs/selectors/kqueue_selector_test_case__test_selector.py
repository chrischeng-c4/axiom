# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "kqueue_selector_test_case__test_selector"
# subject = "cpython.test_selectors.KqueueSelectorTestCase.test_selector"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_selectors.py::KqueueSelectorTestCase::test_selector
"""Auto-ported test: KqueueSelectorTestCase::test_selector (CPython 3.12 oracle)."""


import errno
import os
import random
import selectors
import signal
import socket
import sys
from test import support
from test.support import os_helper
from test.support import socket_helper
from time import sleep
import unittest
import unittest.mock
import tempfile
from time import monotonic as time


try:
    import resource
except ImportError:
    resource = None

if support.is_emscripten or support.is_wasi:
    raise unittest.SkipTest('Cannot create socketpair on Emscripten/WASI.')

if hasattr(socket, 'socketpair'):
    socketpair = socket.socketpair
else:

    def socketpair(family=socket.AF_INET, type=socket.SOCK_STREAM, proto=0):
        with socket.socket(family, type, proto) as l:
            l.bind((socket_helper.HOST, 0))
            l.listen()
            c = socket.socket(family, type, proto)
            try:
                c.connect(l.getsockname())
                caddr = c.getsockname()
                while True:
                    a, addr = l.accept()
                    if addr == caddr:
                        return (c, a)
                    a.close()
            except OSError:
                c.close()
                raise

def find_ready_matching(ready, flag):
    match = []
    for key, events in ready:
        if events & flag:
            match.append(key.fileobj)
    return match

def tearDownModule():
    support.reap_children()


# --- test body ---
SELECTOR = getattr(selectors, 'KqueueSelector', None)

def make_socketpair():
    rd, wr = socketpair()
    pass
    pass
    return (rd, wr)
s = SELECTOR()
pass
NUM_SOCKETS = 12
MSG = b' This is a test.'
MSG_LEN = len(MSG)
readers = []
writers = []
r2w = {}
w2r = {}
for i in range(NUM_SOCKETS):
    rd, wr = make_socketpair()
    s.register(rd, selectors.EVENT_READ)
    s.register(wr, selectors.EVENT_WRITE)
    readers.append(rd)
    writers.append(wr)
    r2w[rd] = wr
    w2r[wr] = rd
bufs = []
while writers:
    ready = s.select()
    ready_writers = find_ready_matching(ready, selectors.EVENT_WRITE)
    if not ready_writers:

        raise AssertionError('no sockets ready for writing')
    wr = random.choice(ready_writers)
    wr.send(MSG)
    for i in range(10):
        ready = s.select()
        ready_readers = find_ready_matching(ready, selectors.EVENT_READ)
        if ready_readers:
            break
        sleep(0.1)
    else:

        raise AssertionError('no sockets ready for reading')

    assert [w2r[wr]] == ready_readers
    rd = ready_readers[0]
    buf = rd.recv(MSG_LEN)

    assert len(buf) == MSG_LEN
    bufs.append(buf)
    s.unregister(r2w[rd])
    s.unregister(rd)
    writers.remove(r2w[rd])

assert bufs == [MSG] * NUM_SOCKETS
print("KqueueSelectorTestCase::test_selector: ok")
