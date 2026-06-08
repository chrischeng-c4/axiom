# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "kqueue"
# dimension = "behavior"
# case = "test_k_queue__test_create_queue"
# subject = "cpython.test_kqueue.TestKQueue.test_create_queue"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_kqueue.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_kqueue.py::TestKQueue::test_create_queue
"""Auto-ported test: TestKQueue::test_create_queue (CPython 3.12 oracle)."""


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

assert kq.fileno() > 0

assert not kq.closed
kq.close()

assert kq.closed

try:
    kq.fileno()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestKQueue::test_create_queue: ok")
