# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "raise_signal_test__test_thread_interrupt_main"
# subject = "cpython.test_signal.RaiseSignalTest.test__thread_interrupt_main"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::RaiseSignalTest::test__thread_interrupt_main
"""Auto-ported test: RaiseSignalTest::test__thread_interrupt_main (CPython 3.12 oracle)."""


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
code = 'if 1:\n        import _thread\n        class Foo():\n            def __del__(self):\n                _thread.interrupt_main()\n\n        x = Foo()\n        '
rc, out, err = assert_python_ok('-c', code)

assert b'OSError: Signal 2 ignored due to race condition' in err
print("RaiseSignalTest::test__thread_interrupt_main: ok")
