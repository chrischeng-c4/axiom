# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fcntl"
# dimension = "behavior"
# case = "test_fcntl__test_fcntl_file_descriptor"
# subject = "cpython.test_fcntl.TestFcntl.test_fcntl_file_descriptor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fcntl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fcntl.py::TestFcntl::test_fcntl_file_descriptor
"""Auto-ported test: TestFcntl::test_fcntl_file_descriptor (CPython 3.12 oracle)."""


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
def get_lockdata():
    try:
        os.O_LARGEFILE
    except AttributeError:
        start_len = 'll'
    else:
        start_len = 'qq'
    if sys.platform.startswith(('netbsd', 'freebsd', 'openbsd')) or sys.platform == 'darwin':
        if struct.calcsize('l') == 8:
            off_t = 'l'
            pid_t = 'i'
        else:
            off_t = 'lxxxx'
            pid_t = 'l'
        lockdata = struct.pack(off_t + off_t + pid_t + 'hh', 0, 0, 0, fcntl.F_WRLCK, 0)
    elif sys.platform.startswith('gnukfreebsd'):
        lockdata = struct.pack('qqihhi', 0, 0, 0, fcntl.F_WRLCK, 0, 0)
    elif sys.platform in ['hp-uxB', 'unixware7']:
        lockdata = struct.pack('hhlllii', fcntl.F_WRLCK, 0, 0, 0, 0, 0, 0)
    else:
        lockdata = struct.pack('hh' + start_len + 'hh', fcntl.F_WRLCK, 0, 0, 0, 0, 0)
    if lockdata:
        if verbose:
            print('struct.pack: ', repr(lockdata))
    return lockdata
self_f = None
self_f = open(TESTFN, 'wb')
rv = fcntl.fcntl(self_f, fcntl.F_SETFL, os.O_NONBLOCK)
if verbose:
    print('Status from fcntl with O_NONBLOCK: ', rv)
lockdata = get_lockdata()
rv = fcntl.fcntl(self_f, fcntl.F_SETLKW, lockdata)
if verbose:
    print('String from fcntl with F_SETLKW: ', repr(rv))
self_f.close()
print("TestFcntl::test_fcntl_file_descriptor: ok")
