# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "itimer_test__test_itimer_exc"
# subject = "cpython.test_signal.ItimerTest.test_itimer_exc"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::ItimerTest::test_itimer_exc
"""Auto-ported test: ItimerTest::test_itimer_exc (CPython 3.12 oracle)."""


import enum
import errno
import functools
import inspect
import os
import random
import signal
import socket
import statistics
import subprocess
import sys
import threading
import time
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, spawn_python
from test.support import threading_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

def tearDownModule():
    support.reap_children()


# --- test body ---
def sig_alrm(*args):
    self_hndl_called = True

def sig_prof(*args):
    self_hndl_called = True
    signal.setitimer(signal.ITIMER_PROF, 0)

def sig_vtalrm(*args):
    self_hndl_called = True
    if self_hndl_count > 3:
        raise signal.ItimerError("setitimer didn't disable ITIMER_VIRTUAL timer.")
    elif self_hndl_count == 3:
        signal.setitimer(signal.ITIMER_VIRTUAL, 0)
    self_hndl_count += 1
self_hndl_called = False
self_hndl_count = 0
self_itimer = None
self_old_alarm = signal.signal(signal.SIGALRM, sig_alrm)

try:
    signal.setitimer(-1, 0)
    raise AssertionError('expected signal.ItimerError')
except signal.ItimerError:
    pass
if 0:

    try:
        signal.setitimer(signal.ITIMER_REAL, -1)
        raise AssertionError('expected signal.ItimerError')
    except signal.ItimerError:
        pass
print("ItimerTest::test_itimer_exc: ok")
