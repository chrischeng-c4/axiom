# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fcntl"
# dimension = "behavior"
# case = "test_fcntl__test_flock"
# subject = "cpython.test_fcntl.TestFcntl.test_flock"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fcntl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fcntl.py::TestFcntl::test_flock
"""Auto-ported test: TestFcntl::test_flock (CPython 3.12 oracle)."""


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
self_f = open(TESTFN, 'wb+')
fileno = self_f.fileno()
fcntl.flock(fileno, fcntl.LOCK_SH)
fcntl.flock(fileno, fcntl.LOCK_UN)
fcntl.flock(self_f, fcntl.LOCK_SH | fcntl.LOCK_NB)
fcntl.flock(self_f, fcntl.LOCK_UN)
fcntl.flock(fileno, fcntl.LOCK_EX)
fcntl.flock(fileno, fcntl.LOCK_UN)

try:
    fcntl.flock(-1, fcntl.LOCK_SH)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    fcntl.flock('spam', fcntl.LOCK_SH)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestFcntl::test_flock: ok")
