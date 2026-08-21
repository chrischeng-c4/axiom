# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "posix_tests__test_setting_signal_handler_to_none_raises_error"
# subject = "cpython.test_signal.PosixTests.test_setting_signal_handler_to_none_raises_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_signal.py::PosixTests::test_setting_signal_handler_to_none_raises_error
"""Auto-ported test: PosixTests::test_setting_signal_handler_to_none_raises_error (CPython 3.12 oracle)."""


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

try:
    signal.signal(signal.SIGUSR1, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PosixTests::test_setting_signal_handler_to_none_raises_error: ok")
