# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "siginterrupt_test__test_siginterrupt_on"
# subject = "cpython.test_signal.SiginterruptTest.test_siginterrupt_on"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::SiginterruptTest::test_siginterrupt_on
"""Auto-ported test: SiginterruptTest::test_siginterrupt_on (CPython 3.12 oracle)."""


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
def readpipe_interrupted(interrupt, timeout=support.SHORT_TIMEOUT):
    """Perform a read during which a signal will arrive.  Return True if the
        read is interrupted by the signal and raises an exception.  Return False
        if it returns normally.
        """
    code = 'if 1:\n            import errno\n            import os\n            import signal\n            import sys\n\n            interrupt = %r\n            r, w = os.pipe()\n\n            def handler(signum, frame):\n                1 / 0\n\n            signal.signal(signal.SIGALRM, handler)\n            if interrupt is not None:\n                signal.siginterrupt(signal.SIGALRM, interrupt)\n\n            print("ready")\n            sys.stdout.flush()\n\n            # run the test twice\n            try:\n                for loop in range(2):\n                    # send a SIGALRM in a second (during the read)\n                    signal.alarm(1)\n                    try:\n                        # blocking call: read from a pipe without data\n                        os.read(r, 1)\n                    except ZeroDivisionError:\n                        pass\n                    else:\n                        sys.exit(2)\n                sys.exit(3)\n            finally:\n                os.close(r)\n                os.close(w)\n        ' % (interrupt,)
    with spawn_python('-c', code) as process:
        try:
            first_line = process.stdout.readline()
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.kill()
            return False
        else:
            stdout = first_line + stdout
            exitcode = process.wait()
            if exitcode not in (2, 3):
                raise Exception('Child error (exit code %s): %r' % (exitcode, stdout))
            return exitcode == 3
interrupted = readpipe_interrupted(True)

assert interrupted
print("SiginterruptTest::test_siginterrupt_on: ok")
