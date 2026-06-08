# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "policy_tests__test_get_child_watcher_thread"
# subject = "cpython.test_unix_events.PolicyTests.test_get_child_watcher_thread"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_unix_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unix_events.py::PolicyTests::test_get_child_watcher_thread
"""Auto-ported test: PolicyTests::test_get_child_watcher_thread (CPython 3.12 oracle)."""


import contextlib
import errno
import io
import multiprocessing
from multiprocessing.util import _cleanup_tests as multiprocessing_cleanup_tests
import os
import signal
import socket
import stat
import sys
import threading
import time
import unittest
from unittest import mock
import warnings
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import wait_process
from test.support import hashlib_helper
import asyncio
from asyncio import log
from asyncio import unix_events
from test.test_asyncio import utils as test_utils


'Tests for unix_events.py.'

if sys.platform == 'win32':
    raise unittest.SkipTest('UNIX only')

def tearDownModule():
    asyncio.set_event_loop_policy(None)

MOCK_ANY = mock.ANY

def EXITCODE(exitcode):
    return 32768 + exitcode

def SIGNAL(signum):
    if not 1 <= signum <= 68:
        raise AssertionError(f'invalid signum {signum}')
    return 32768 - signum

def close_pipe_transport(transport):
    if transport._pipe is None:
        return
    transport._pipe.close()
    transport._pipe = None


# --- test body ---
def create_policy():
    return asyncio.DefaultEventLoopPolicy()

def f():
    policy.set_event_loop(policy.new_event_loop())
    self.assertIsInstance(policy.get_event_loop(), asyncio.AbstractEventLoop)
    with warnings.catch_warnings():
        warnings.simplefilter('ignore', DeprecationWarning)
        watcher = policy.get_child_watcher()
    self.assertIsInstance(watcher, asyncio.SafeChildWatcher)
    self.assertIsNone(watcher._loop)
    policy.get_event_loop().close()
policy = create_policy()
with warnings.catch_warnings():
    warnings.simplefilter('ignore', DeprecationWarning)
    policy.set_child_watcher(asyncio.SafeChildWatcher())
th = threading.Thread(target=f)
th.start()
th.join()
print("PolicyTests::test_get_child_watcher_thread: ok")
