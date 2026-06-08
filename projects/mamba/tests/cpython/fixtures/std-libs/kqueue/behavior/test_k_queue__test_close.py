# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "kqueue"
# dimension = "behavior"
# case = "test_k_queue__test_close"
# subject = "cpython.test_kqueue.TestKQueue.test_close"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_kqueue.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_kqueue.py::TestKQueue::test_close
"""Auto-ported test: TestKQueue::test_close (CPython 3.12 oracle)."""


import errno
import os
import select
import socket
from test import support
import time
import unittest


'\nTests for kqueue wrapper.\n'

if not hasattr(select, 'kqueue'):
    raise unittest.SkipTest('test works only on BSD')


# --- test body ---
def testPair():
    kq = select.kqueue()
    a, b = socket.socketpair()
    a.send(b'foo')
    event1 = select.kevent(a, select.KQ_FILTER_READ, select.KQ_EV_ADD | select.KQ_EV_ENABLE)
    event2 = select.kevent(b, select.KQ_FILTER_READ, select.KQ_EV_ADD | select.KQ_EV_ENABLE)
    r = kq.control([event1, event2], 1, 1)

    assert r

    assert not r[0].flags & select.KQ_EV_ERROR

    assert b.recv(r[0].data) == b'foo'
    a.close()
    b.close()
    kq.close()
open_file = open(__file__, 'rb')
pass
fd = open_file.fileno()
kqueue = select.kqueue()

assert isinstance(kqueue.fileno(), int)

assert not kqueue.closed
kqueue.close()

assert kqueue.closed

try:
    kqueue.fileno()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
kqueue.close()

try:
    kqueue.control(None, 4)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestKQueue::test_close: ok")
