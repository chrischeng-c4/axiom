# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "posix_tests__test_no_repr_is_called_on_signal_handler"
# subject = "cpython.test_signal.PosixTests.test_no_repr_is_called_on_signal_handler"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::PosixTests::test_no_repr_is_called_on_signal_handler
"""Auto-ported test: PosixTests::test_no_repr_is_called_on_signal_handler (CPython 3.12 oracle)."""


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

class MyArgument:

    def __init__(self):
        self.repr_count = 0

    def __repr__(self):
        self.repr_count += 1
        return super().__repr__()
argument = MyArgument()

assert 0 == argument.repr_count
handler = create_handler_with_partial(argument)
hup = signal.signal(signal.SIGHUP, handler)

assert isinstance(hup, signal.Handlers)

assert signal.getsignal(signal.SIGHUP) == handler
signal.signal(signal.SIGHUP, hup)

assert signal.getsignal(signal.SIGHUP) == hup

assert 0 == argument.repr_count
print("PosixTests::test_no_repr_is_called_on_signal_handler: ok")
