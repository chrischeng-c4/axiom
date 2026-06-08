# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "wakeup_fd_tests__test_invalid_fd"
# subject = "cpython.test_signal.WakeupFDTests.test_invalid_fd"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::WakeupFDTests::test_invalid_fd
"""Auto-ported test: WakeupFDTests::test_invalid_fd (CPython 3.12 oracle)."""


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
fd = os_helper.make_bad_fd()

try:
    signal.set_wakeup_fd(fd)
    raise AssertionError('expected (ValueError, OSError)')
except (ValueError, OSError):
    pass
print("WakeupFDTests::test_invalid_fd: ok")
