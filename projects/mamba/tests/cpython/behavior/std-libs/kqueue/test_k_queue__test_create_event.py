# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "kqueue"
# dimension = "behavior"
# case = "test_k_queue__test_create_event"
# subject = "cpython.test_kqueue.TestKQueue.test_create_event"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_kqueue.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_kqueue.py::TestKQueue::test_create_event
"""Auto-ported test: TestKQueue::test_create_event (CPython 3.12 oracle)."""


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
from operator import lt, le, gt, ge
fd = os.open(os.devnull, os.O_WRONLY)
pass
ev = select.kevent(fd)
other = select.kevent(1000)

assert ev.ident == fd

assert ev.filter == select.KQ_FILTER_READ

assert ev.flags == select.KQ_EV_ADD

assert ev.fflags == 0

assert ev.data == 0

assert ev.udata == 0

assert ev == ev

assert ev != other

assert ev < other

assert other >= ev
for op in (lt, le, gt, ge):

    try:
        op(ev, None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        op(ev, 1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        op(ev, 'ev')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
ev = select.kevent(fd, select.KQ_FILTER_WRITE)

assert ev.ident == fd

assert ev.filter == select.KQ_FILTER_WRITE

assert ev.flags == select.KQ_EV_ADD

assert ev.fflags == 0

assert ev.data == 0

assert ev.udata == 0

assert ev == ev

assert ev != other
ev = select.kevent(fd, select.KQ_FILTER_WRITE, select.KQ_EV_ONESHOT)

assert ev.ident == fd

assert ev.filter == select.KQ_FILTER_WRITE

assert ev.flags == select.KQ_EV_ONESHOT

assert ev.fflags == 0

assert ev.data == 0

assert ev.udata == 0

assert ev == ev

assert ev != other
ev = select.kevent(1, 2, 3, 4, 5, 6)

assert ev.ident == 1

assert ev.filter == 2

assert ev.flags == 3

assert ev.fflags == 4

assert ev.data == 5

assert ev.udata == 6

assert ev == ev

assert ev != other
bignum = 32767
ev = select.kevent(bignum, 1, 2, 3, bignum - 1, bignum)

assert ev.ident == bignum

assert ev.filter == 1

assert ev.flags == 2

assert ev.fflags == 3

assert ev.data == bignum - 1

assert ev.udata == bignum

assert ev == ev

assert ev != other
bignum = 65535
ev = select.kevent(0, 1, bignum)

assert ev.ident == 0

assert ev.filter == 1

assert ev.flags == bignum

assert ev.fflags == 0

assert ev.data == 0

assert ev.udata == 0

assert ev == ev

assert ev != other
bignum = 4294967295
ev = select.kevent(0, 1, 2, bignum)

assert ev.ident == 0

assert ev.filter == 1

assert ev.flags == 2

assert ev.fflags == bignum

assert ev.data == 0

assert ev.udata == 0

assert ev == ev

assert ev != other
print("TestKQueue::test_create_event: ok")
