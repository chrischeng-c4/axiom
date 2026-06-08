# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pty"
# dimension = "behavior"
# case = "small_pty_tests__test_copy_to_each"
# subject = "cpython.test_pty.SmallPtyTests.test__copy_to_each"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pty.py::SmallPtyTests::test__copy_to_each
"""Auto-ported test: SmallPtyTests::test__copy_to_each (CPython 3.12 oracle)."""


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
def _make_mock_fork(pid):

    def mock_fork():
        return (pid, 12)
    return mock_fork

def _mock_select(rfds, wfds, xfds):

    assert (rfds, wfds, xfds) == self_select_input.pop(0)
    return self_select_output.pop(0)

def _mock_tcsetattr(fileno, opt, mode):
    self_tcsetattr_mode_setting = mode

def _pipe():
    pipe_fds = os.pipe()
    self_fds.extend(pipe_fds)
    return pipe_fds

def _socketpair():
    socketpair = socket.socketpair()
    self_files.extend(socketpair)
    return socketpair
self_orig_stdin_fileno = pty.STDIN_FILENO
self_orig_stdout_fileno = pty.STDOUT_FILENO
self_orig_pty_close = pty.close
self_orig_pty__copy = pty._copy
self_orig_pty_fork = pty.fork
self_orig_pty_select = pty.select
self_orig_pty_setraw = pty.setraw
self_orig_pty_tcgetattr = pty.tcgetattr
self_orig_pty_tcsetattr = pty.tcsetattr
self_orig_pty_waitpid = pty.waitpid
self_fds = []
self_files = []
self_select_input = []
self_select_output = []
self_tcsetattr_mode_setting = None
'Test the normal data case on both master_fd and stdin.'
read_from_stdout_fd, mock_stdout_fd = _pipe()
pty.STDOUT_FILENO = mock_stdout_fd
mock_stdin_fd, write_to_stdin_fd = _pipe()
pty.STDIN_FILENO = mock_stdin_fd
socketpair = _socketpair()
masters = [s.fileno() for s in socketpair]
write_all(masters[1], b'from master')
write_all(write_to_stdin_fd, b'from stdin')
pty.select = _mock_select
self_select_input.append(([mock_stdin_fd, masters[0]], [], []))
self_select_output.append(([mock_stdin_fd, masters[0]], [], []))
self_select_input.append(([mock_stdin_fd, masters[0]], [mock_stdout_fd, masters[0]], []))
self_select_output.append(([], [mock_stdout_fd, masters[0]], []))
self_select_input.append(([mock_stdin_fd, masters[0]], [], []))
try:
    pty._copy(masters[0])
    raise AssertionError('expected IndexError')
except IndexError:
    pass
rfds = select.select([read_from_stdout_fd, masters[1]], [], [], 0)[0]

assert [read_from_stdout_fd, masters[1]] == rfds

assert os.read(read_from_stdout_fd, 20) == b'from master'

assert os.read(masters[1], 20) == b'from stdin'
print("SmallPtyTests::test__copy_to_each: ok")
