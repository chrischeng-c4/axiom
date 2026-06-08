# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pty"
# dimension = "behavior"
# case = "pty_test__test_openpty"
# subject = "cpython.test_pty.PtyTest.test_openpty"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pty.py::PtyTest::test_openpty
"""Auto-ported test: PtyTest::test_openpty (CPython 3.12 oracle)."""


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
try:
    mode = tty.tcgetattr(pty.STDIN_FILENO)
except tty.error:
    debug('tty.tcgetattr(pty.STDIN_FILENO) failed')
    mode = None
new_dim = None
if self_stdin_dim:
    try:
        debug('Setting pty.STDIN_FILENO window size.')
        debug(f'original size: (row, col) = {self_stdin_dim}')
        target_dim = (self_stdin_dim[0] + 1, self_stdin_dim[1] + 1)
        debug(f'target size: (row, col) = {target_dim}')
        tty.tcsetwinsize(pty.STDIN_FILENO, target_dim)
        new_dim = tty.tcgetwinsize(pty.STDIN_FILENO)

        assert new_dim == target_dim
    except OSError:
        warnings.warn('Failed to set pty.STDIN_FILENO window size.')
        pass
try:
    debug('Calling pty.openpty()')
    try:
        master_fd, slave_fd, slave_name = pty.openpty(mode, new_dim, True)
    except TypeError:
        master_fd, slave_fd = pty.openpty()
        slave_name = None
    debug(f'Got master_fd={master_fd!r}, slave_fd={slave_fd!r}, slave_name={slave_name!r}')
except OSError:
    raise unittest.SkipTest('Pseudo-terminals (seemingly) not functional.')
pass
pass

assert os.isatty(slave_fd)
if mode:

    assert tty.tcgetattr(slave_fd) == mode
if new_dim:

    assert tty.tcgetwinsize(slave_fd) == new_dim
blocking = os.get_blocking(master_fd)
try:
    os.set_blocking(master_fd, False)
    try:
        s1 = os.read(master_fd, 1024)

        assert b'' == s1
    except OSError as e:
        if e.errno != errno.EAGAIN:
            raise
finally:
    os.set_blocking(master_fd, blocking)
debug('Writing to slave_fd')
write_all(slave_fd, TEST_STRING_1)
s1 = _readline(master_fd)

assert b'I wish to buy a fish license.\n' == normalize_output(s1)
debug('Writing chunked output')
write_all(slave_fd, TEST_STRING_2[:5])
write_all(slave_fd, TEST_STRING_2[5:])
s2 = _readline(master_fd)

assert b'For my pet fish, Eric.\n' == normalize_output(s2)
print("PtyTest::test_openpty: ok")
