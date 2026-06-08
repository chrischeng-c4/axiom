# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wait3"
# dimension = "behavior"
# case = "wait3_test__test_wait3_rusage_initialized"
# subject = "cpython.test_wait3.Wait3Test.test_wait3_rusage_initialized"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_wait3.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_wait3.py::Wait3Test::test_wait3_rusage_initialized
"""Auto-ported test: Wait3Test::test_wait3_rusage_initialized (CPython 3.12 oracle)."""


import os
import subprocess
import sys
import unittest
from test.fork_wait import ForkWait
from test import support


'This test checks for correct wait3() behavior.\n'

if not support.has_fork_support:
    raise unittest.SkipTest('requires working os.fork()')

if not hasattr(os, 'wait3'):
    raise unittest.SkipTest('os.wait3 not defined')

def tearDownModule():
    support.reap_children()


# --- test body ---
def wait_impl(cpid, *, exitcode):
    for _ in support.sleeping_retry(support.SHORT_TIMEOUT):
        spid, status, rusage = os.wait3(os.WNOHANG)
        if spid == cpid:
            break

    assert spid == cpid

    assert os.waitstatus_to_exitcode(status) == exitcode

    assert rusage
args = [sys.executable, '-c', 'import sys; sys.stdin.read()']
proc = subprocess.Popen(args, stdin=subprocess.PIPE)
try:
    pid, status, rusage = os.wait3(os.WNOHANG)

    assert 0 == pid

    assert 0 == status

    assert 0 == sum(rusage)
finally:
    proc.stdin.close()
    proc.wait()
print("Wait3Test::test_wait3_rusage_initialized: ok")
