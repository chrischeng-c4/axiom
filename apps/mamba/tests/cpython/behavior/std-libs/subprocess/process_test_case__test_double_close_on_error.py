# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "process_test_case__test_double_close_on_error"
# subject = "cpython.test_subprocess.ProcessTestCase.test_double_close_on_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subprocess.py::ProcessTestCase::test_double_close_on_error
"""Auto-ported test: ProcessTestCase::test_double_close_on_error (CPython 3.12 oracle)."""


import unittest
from unittest import mock
from test import support
from test.support import check_sanitizer
from test.support import import_helper
from test.support import os_helper
from test.support import warnings_helper
from test.support.script_helper import assert_python_ok
import subprocess
import sys
import signal
import io
import itertools
import os
import errno
import tempfile
import time
import traceback
import types
import selectors
import sysconfig
import select
import shutil
import threading
import gc
import textwrap
import json
from test.support.os_helper import FakePath


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import pwd
except ImportError:
    pwd = None

try:
    import grp
except ImportError:
    grp = None

try:
    import fcntl
except:
    fcntl = None

if support.PGO:
    raise unittest.SkipTest('test is not helpful for PGO')

if not support.has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

mswindows = sys.platform == 'win32'

if mswindows:
    SETBINARY = 'import msvcrt; msvcrt.setmode(sys.stdout.fileno(), os.O_BINARY);'
else:
    SETBINARY = ''

NONEXISTING_CMD = ('nonexisting_i_hope',)

NONEXISTING_ERRORS = (FileNotFoundError, NotADirectoryError, PermissionError)

ZERO_RETURN_CMD = (sys.executable, '-c', 'pass')

def setUpModule():
    shell_true = shutil.which('true')
    if shell_true is None:
        return
    if os.access(shell_true, os.X_OK) and subprocess.run([shell_true]).returncode == 0:
        global ZERO_RETURN_CMD
        ZERO_RETURN_CMD = (shell_true,)

class PopenTestException(Exception):
    pass

class PopenExecuteChildRaises(subprocess.Popen):
    """Popen subclass for testing cleanup of subprocess.PIPE filehandles when
    _execute_child fails.
    """

    def _execute_child(self, *args, **kwargs):
        raise PopenTestException('Forced Exception for Test')

def _get_test_grp_name():
    for name_group in ('staff', 'nogroup', 'grp', 'nobody', 'nfsnobody'):
        if grp:
            try:
                grp.getgrnam(name_group)
            except KeyError:
                continue
            return name_group
    else:
        raise unittest.SkipTest('No identified group name to use for this test on this platform.')


# --- test body ---
def _assert_cwd(expected_cwd, python_arg, **kwargs):
    p = subprocess.Popen([python_arg, '-c', 'import os, sys; buf = sys.stdout.buffer; buf.write(os.getcwd().encode()); buf.flush(); sys.exit(47)'], stdout=subprocess.PIPE, **kwargs)
    pass
    p.wait()

    assert 47 == p.returncode
    normcase = os.path.normcase

    assert normcase(expected_cwd) == normcase(p.stdout.read().decode())

def _assert_python(pre_args, **kwargs):
    args = pre_args + ['import sys; sys.exit(47)']
    p = subprocess.Popen(args, **kwargs)
    p.wait()

    assert 47 == p.returncode

def _normalize_cwd(cwd):
    with os_helper.change_cwd(cwd):
        return os.getcwd()

def _split_python_path():
    python_path = os.path.realpath(sys.executable)
    return os.path.split(python_path)

def _test_bufsize_equal_one(line, expected, universal_newlines):
    with subprocess.Popen([sys.executable, '-c', 'import sys;sys.stdout.write(sys.stdin.readline());sys.stdout.flush()'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, bufsize=1, universal_newlines=universal_newlines) as p:
        p.stdin.write(line)
        os.close(p.stdin.fileno())
        read_line = p.stdout.readline()
        with support.SuppressCrashReport():
            try:
                p.stdin.close()
            except OSError:
                pass
        p.stdin = None

    assert p.returncode == 0

    assert read_line == expected
support.reap_children()
fds = []

def open_fds():
    for i in range(20):
        fds.extend(os.pipe())
        time.sleep(0.001)
t = threading.Thread(target=open_fds)
t.start()
try:
    try:
        subprocess.Popen(NONEXISTING_CMD, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        raise AssertionError('expected OSError')
    except OSError:
        pass
finally:
    t.join()
    exc = None
    for fd in fds:
        try:
            os.close(fd)
        except OSError as e:
            exc = e
    if exc is not None:
        raise exc
print("ProcessTestCase::test_double_close_on_error: ok")
