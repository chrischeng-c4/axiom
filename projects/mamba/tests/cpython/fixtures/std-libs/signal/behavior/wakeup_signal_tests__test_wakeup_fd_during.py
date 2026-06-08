# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "wakeup_signal_tests__test_wakeup_fd_during"
# subject = "cpython.test_signal.WakeupSignalTests.test_wakeup_fd_during"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::WakeupSignalTests::test_wakeup_fd_during
"""Auto-ported test: WakeupSignalTests::test_wakeup_fd_during (CPython 3.12 oracle)."""


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
check_wakeup('def test():\n            import select\n            import time\n\n            TIMEOUT_FULL = 10\n            TIMEOUT_HALF = 5\n\n            class InterruptSelect(Exception):\n                pass\n\n            def handler(signum, frame):\n                raise InterruptSelect\n            signal.signal(signal.SIGALRM, handler)\n\n            signal.alarm(1)\n            before_time = time.monotonic()\n            # We attempt to get a signal during the select call\n            try:\n                select.select([read], [], [], TIMEOUT_FULL)\n            except InterruptSelect:\n                pass\n            else:\n                raise Exception("select() was not interrupted")\n            after_time = time.monotonic()\n            dt = after_time - before_time\n            if dt >= TIMEOUT_HALF:\n                raise Exception("%s >= %s" % (dt, TIMEOUT_HALF))\n        ', signal.SIGALRM)
print("WakeupSignalTests::test_wakeup_fd_during: ok")
