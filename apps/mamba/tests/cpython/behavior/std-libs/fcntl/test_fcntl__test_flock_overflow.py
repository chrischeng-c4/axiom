# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fcntl"
# dimension = "behavior"
# case = "test_fcntl__test_flock_overflow"
# subject = "cpython.test_fcntl.TestFcntl.test_flock_overflow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fcntl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fcntl.py::TestFcntl::test_flock_overflow
"""Auto-ported test: TestFcntl::test_flock_overflow (CPython 3.12 oracle)."""


import multiprocessing
import platform
import os
import struct
import sys
import unittest
from test.support import verbose, cpython_only, get_pagesize
from test.support.import_helper import import_module
from test.support.os_helper import TESTFN, unlink


'Test program for the fcntl C module.\n'

fcntl = import_module('fcntl')

class BadFile:

    def __init__(self, fn):
        self.fn = fn

    def fileno(self):
        return self.fn

def try_lockf_on_other_process_fail(fname, cmd):
    f = open(fname, 'wb+')
    try:
        fcntl.lockf(f, cmd)
    except BlockingIOError:
        pass
    finally:
        f.close()

def try_lockf_on_other_process(fname, cmd):
    f = open(fname, 'wb+')
    fcntl.lockf(f, cmd)
    fcntl.lockf(f, fcntl.LOCK_UN)
    f.close()


# --- test body ---
self_f = None
import _testcapi

try:
    fcntl.flock(_testcapi.INT_MAX + 1, fcntl.LOCK_SH)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass
print("TestFcntl::test_flock_overflow: ok")
