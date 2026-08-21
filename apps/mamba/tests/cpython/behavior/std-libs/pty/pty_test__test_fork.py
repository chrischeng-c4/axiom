# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pty"
# dimension = "behavior"
# case = "pty_test__test_fork"
# subject = "cpython.test_pty.PtyTest.test_fork"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pty.py::PtyTest::test_fork
"""Auto-ported test: PtyTest::test_fork (CPython 3.12 oracle)."""


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
debug('calling pty.fork()')
pid, master_fd = pty.fork()
pass
if pid == pty.CHILD:
    if not os.isatty(1):
        debug("Child's fd 1 is not a tty?!")
        os._exit(3)
    debug('In child, calling os.setsid()')
    try:
        os.setsid()
    except OSError:
        debug('Good: OSError was raised.')
        pass
    except AttributeError:
        debug('No setsid() available?')
        pass
    except:
        debug('An unexpected error was raised.')
        os._exit(1)
    else:
        debug('os.setsid() succeeded! (bad!)')
        os._exit(2)
    os._exit(4)
else:
    debug('Waiting for child (%d) to finish.' % pid)
    while True:
        try:
            data = os.read(master_fd, 80)
        except OSError:
            break
        if not data:
            break
        sys.stdout.write(str(data.replace(b'\r\n', b'\n'), encoding='ascii'))
    pid, status = os.waitpid(pid, 0)
    res = os.waitstatus_to_exitcode(status)
    debug('Child (%d) exited with code %d (status %d).' % (pid, res, status))
    if res == 1:

        raise AssertionError('Child raised an unexpected exception in os.setsid()')
    elif res == 2:

        raise AssertionError('pty.fork() failed to make child a session leader.')
    elif res == 3:

        raise AssertionError('Child spawned by pty.fork() did not have a tty as stdout')
    elif res != 4:

        raise AssertionError('pty.fork() failed for unknown reasons.')
print("PtyTest::test_fork: ok")
