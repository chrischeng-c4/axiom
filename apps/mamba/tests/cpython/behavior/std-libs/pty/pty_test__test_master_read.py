# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pty"
# dimension = "behavior"
# case = "pty_test__test_master_read"
# subject = "cpython.test_pty.PtyTest.test_master_read"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pty.py::PtyTest::test_master_read
"""Auto-ported test: PtyTest::test_master_read (CPython 3.12 oracle)."""


from test.support import verbose, reap_children
from test.support.os_helper import TESTFN, unlink
from test.support.import_helper import import_module
import errno
import os
import pty
import tty
import sys
import select
import signal
import socket
import io
import unittest
import warnings


import_module('termios')

import_module('fcntl')

TEST_STRING_1 = b'I wish to buy a fish license.\n'

TEST_STRING_2 = b'For my pet fish, Eric.\n'

_HAVE_WINSZ = hasattr(tty, 'TIOCGWINSZ') and hasattr(tty, 'TIOCSWINSZ')

if verbose:

    def debug(msg):
        print(msg)
else:

    def debug(msg):
        pass

def normalize_output(data):
    if data.endswith(b'\r\r\n'):
        return data.replace(b'\r\r\n', b'\n')
    if data.endswith(b'\r\n'):
        return data.replace(b'\r\n', b'\n')
    return data

def _readline(fd):
    """Read one line.  May block forever if no newline is read."""
    reader = io.FileIO(fd, mode='rb', closefd=False)
    return reader.readline()

def expectedFailureIfStdinIsTTY(fun):
    try:
        tty.tcgetattr(pty.STDIN_FILENO)
        return unittest.expectedFailure(fun)
    except tty.error:
        pass
    return fun

def write_all(fd, data):
    written = os.write(fd, data)
    if written != len(data):
        raise Exception(f'short write: os.write({fd}, {len(data)} bytes) wrote {written} bytes')

def tearDownModule():
    reap_children()


# --- test body ---
def handle_sighup(signum, frame):
    pass
old_sighup = signal.signal(signal.SIGHUP, handle_sighup)
pass
self_stdin_dim = None
if _HAVE_WINSZ:
    try:
        self_stdin_dim = tty.tcgetwinsize(pty.STDIN_FILENO)
        pass
    except tty.error:
        pass
debug('Calling pty.openpty()')
master_fd, slave_fd = pty.openpty()
debug(f"Got master_fd '{master_fd}', slave_fd '{slave_fd}'")
pass
debug('Closing slave_fd')
os.close(slave_fd)
debug('Reading from master_fd')
try:
    data = os.read(master_fd, 1)
except OSError:
    data = b''

assert data == b''
print("PtyTest::test_master_read: ok")
