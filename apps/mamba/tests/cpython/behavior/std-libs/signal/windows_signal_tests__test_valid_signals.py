# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "windows_signal_tests__test_valid_signals"
# subject = "cpython.test_signal.WindowsSignalTests.test_valid_signals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::WindowsSignalTests::test_valid_signals
"""Auto-ported test: WindowsSignalTests::test_valid_signals (CPython 3.12 oracle)."""


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
s = signal.valid_signals()

assert isinstance(s, set)

assert len(s) >= 6

assert signal.Signals.SIGINT in s

assert 0 not in s

assert signal.NSIG not in s

assert len(s) < signal.NSIG
print("WindowsSignalTests::test_valid_signals: ok")
