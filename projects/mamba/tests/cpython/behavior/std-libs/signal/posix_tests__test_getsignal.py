# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "posix_tests__test_getsignal"
# subject = "cpython.test_signal.PosixTests.test_getsignal"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::PosixTests::test_getsignal
"""Auto-ported test: PosixTests::test_getsignal (CPython 3.12 oracle)."""


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
def create_handler_with_partial(argument):
    return functools.partial(trivial_signal_handler, argument)

def trivial_signal_handler(*args):
    pass
hup = signal.signal(signal.SIGHUP, trivial_signal_handler)

assert isinstance(hup, signal.Handlers)

assert signal.getsignal(signal.SIGHUP) == trivial_signal_handler
signal.signal(signal.SIGHUP, hup)

assert signal.getsignal(signal.SIGHUP) == hup
print("PosixTests::test_getsignal: ok")
