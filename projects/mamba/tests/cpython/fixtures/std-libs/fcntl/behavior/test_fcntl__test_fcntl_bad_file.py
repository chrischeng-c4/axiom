# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fcntl"
# dimension = "behavior"
# case = "test_fcntl__test_fcntl_bad_file"
# subject = "cpython.test_fcntl.TestFcntl.test_fcntl_bad_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fcntl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fcntl.py::TestFcntl::test_fcntl_bad_file
"""Auto-ported test: TestFcntl::test_fcntl_bad_file (CPython 3.12 oracle)."""


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
try:
    fcntl.fcntl(-1, fcntl.F_SETFL, os.O_NONBLOCK)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    fcntl.fcntl(BadFile(-1), fcntl.F_SETFL, os.O_NONBLOCK)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    fcntl.fcntl('spam', fcntl.F_SETFL, os.O_NONBLOCK)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    fcntl.fcntl(BadFile('spam'), fcntl.F_SETFL, os.O_NONBLOCK)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestFcntl::test_fcntl_bad_file: ok")
