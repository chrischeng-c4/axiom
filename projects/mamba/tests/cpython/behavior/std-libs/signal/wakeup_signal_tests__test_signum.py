# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "wakeup_signal_tests__test_signum"
# subject = "cpython.test_signal.WakeupSignalTests.test_signum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_signal.py::WakeupSignalTests::test_signum
"""Auto-ported test: WakeupSignalTests::test_signum (CPython 3.12 oracle)."""


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
def check_wakeup(test_body, *signals, ordered=True):
    code = 'if 1:\n        import _testcapi\n        import os\n        import signal\n        import struct\n\n        signals = {!r}\n\n        def handler(signum, frame):\n            pass\n\n        def check_signum(signals):\n            data = os.read(read, len(signals)+1)\n            raised = struct.unpack(\'%uB\' % len(data), data)\n            if not {!r}:\n                raised = set(raised)\n                signals = set(signals)\n            if raised != signals:\n                raise Exception("%r != %r" % (raised, signals))\n\n        {}\n\n        signal.signal(signal.SIGALRM, handler)\n        read, write = os.pipe()\n        os.set_blocking(write, False)\n        signal.set_wakeup_fd(write)\n\n        test()\n        check_signum(signals)\n\n        os.close(read)\n        os.close(write)\n        '.format(tuple(map(int, signals)), ordered, test_body)
    assert_python_ok('-c', code)
check_wakeup('def test():\n            signal.signal(signal.SIGUSR1, handler)\n            signal.raise_signal(signal.SIGUSR1)\n            signal.raise_signal(signal.SIGALRM)\n        ', signal.SIGUSR1, signal.SIGALRM)
print("WakeupSignalTests::test_signum: ok")
