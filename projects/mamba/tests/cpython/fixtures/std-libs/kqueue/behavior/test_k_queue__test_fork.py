# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "kqueue"
# dimension = "behavior"
# case = "test_k_queue__test_fork"
# subject = "cpython.test_kqueue.TestKQueue.test_fork"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_kqueue.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_kqueue.py::TestKQueue::test_fork
"""Auto-ported test: TestKQueue::test_fork (CPython 3.12 oracle)."""


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
kqueue = select.kqueue()
if (pid := os.fork()) == 0:
    try:

        assert kqueue.closed
        try:
            kqueue.fileno()
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('closed kqueue', str(_aR_e))
    except:
        os._exit(1)
    finally:
        os._exit(0)
else:
    support.wait_process(pid, exitcode=0)

    assert not kqueue.closed
print("TestKQueue::test_fork: ok")
