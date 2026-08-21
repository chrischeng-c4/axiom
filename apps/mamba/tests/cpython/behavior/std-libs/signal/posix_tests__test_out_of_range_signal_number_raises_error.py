# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "posix_tests__test_out_of_range_signal_number_raises_error"
# subject = "cpython.test_signal.PosixTests.test_out_of_range_signal_number_raises_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::PosixTests::test_out_of_range_signal_number_raises_error
"""Auto-ported test: PosixTests::test_out_of_range_signal_number_raises_error (CPython 3.12 oracle)."""


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

try:
    signal.getsignal(4242)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    signal.signal(4242, trivial_signal_handler)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    signal.strsignal(4242)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PosixTests::test_out_of_range_signal_number_raises_error: ok")
