# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "kqueue"
# dimension = "behavior"
# case = "test_k_queue__test_issue30058"
# subject = "cpython.test_kqueue.TestKQueue.test_issue30058"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_kqueue.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_kqueue.py::TestKQueue::test_issue30058
"""Auto-ported test: TestKQueue::test_issue30058 (CPython 3.12 oracle)."""


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
kq = select.kqueue()
a, b = socket.socketpair()
ev = select.kevent(a, select.KQ_FILTER_READ, select.KQ_EV_ADD | select.KQ_EV_ENABLE)
kq.control([ev], 0)
kq.control((ev,), 0)

class BadList:

    def __len__(self):
        return 0

    def __iter__(self):
        for i in range(100):
            yield ev
kq.control(BadList(), 0)
kq.control(iter([ev]), 0)
a.close()
b.close()
kq.close()
print("TestKQueue::test_issue30058: ok")
