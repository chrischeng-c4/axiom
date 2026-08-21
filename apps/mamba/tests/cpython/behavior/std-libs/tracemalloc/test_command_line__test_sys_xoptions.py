# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "test_command_line__test_sys_xoptions"
# subject = "cpython.test_tracemalloc.TestCommandLine.test_sys_xoptions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tracemalloc.py::TestCommandLine::test_sys_xoptions
"""Auto-ported test: TestCommandLine::test_sys_xoptions (CPython 3.12 oracle)."""


import contextlib
import os
import sys
import textwrap
import tracemalloc
import unittest
from unittest.mock import patch
from test.support.script_helper import assert_python_ok, assert_python_failure, interpreter_requires_environment
from test import support
from test.support import os_helper
from test.support import threading_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

DEFAULT_DOMAIN = 0

EMPTY_STRING_SIZE = sys.getsizeof(b'')

INVALID_NFRAME = (-1, 2 ** 30)

def get_frames(nframe, lineno_delta):
    frames = []
    frame = sys._getframe(1)
    for index in range(nframe):
        code = frame.f_code
        lineno = frame.f_lineno + lineno_delta
        frames.append((code.co_filename, lineno))
        lineno_delta = 0
        frame = frame.f_back
        if frame is None:
            break
    return tuple(frames)

def allocate_bytes(size):
    nframe = tracemalloc.get_traceback_limit()
    bytes_len = size - EMPTY_STRING_SIZE
    frames = get_frames(nframe, 1)
    data = b'x' * bytes_len
    return (data, tracemalloc.Traceback(frames, min(len(frames), nframe)))

def create_snapshots():
    traceback_limit = 2
    raw_traces = [(0, 10, (('a.py', 2), ('b.py', 4)), 3), (0, 10, (('a.py', 2), ('b.py', 4)), 3), (0, 10, (('a.py', 2), ('b.py', 4)), 3), (1, 2, (('a.py', 5), ('b.py', 4)), 3), (2, 66, (('b.py', 1),), 1), (3, 7, (('<unknown>', 0),), 1)]
    snapshot = tracemalloc.Snapshot(raw_traces, traceback_limit)
    raw_traces2 = [(0, 10, (('a.py', 2), ('b.py', 4)), 3), (0, 10, (('a.py', 2), ('b.py', 4)), 3), (0, 10, (('a.py', 2), ('b.py', 4)), 3), (2, 2, (('a.py', 5), ('b.py', 4)), 3), (2, 5000, (('a.py', 5), ('b.py', 4)), 3), (4, 400, (('c.py', 578),), 1)]
    snapshot2 = tracemalloc.Snapshot(raw_traces2, traceback_limit)
    return (snapshot, snapshot2)

def frame(filename, lineno):
    return tracemalloc._Frame((filename, lineno))

def traceback(*frames):
    return tracemalloc.Traceback(frames)

def traceback_lineno(filename, lineno):
    return traceback((filename, lineno))

def traceback_filename(filename):
    return traceback_lineno(filename, 0)


# --- test body ---
def check_env_var_invalid(nframe):
    with support.SuppressCrashReport():
        ok, stdout, stderr = assert_python_failure('-c', 'pass', PYTHONTRACEMALLOC=str(nframe))
    if b'ValueError: the number of frames must be in range' in stderr:
        return
    if b'PYTHONTRACEMALLOC: invalid number of frames' in stderr:
        return

    raise AssertionError(f'unexpected output: {stderr!a}')

def check_sys_xoptions_invalid(nframe):
    args = ('-X', 'tracemalloc=%s' % nframe, '-c', 'pass')
    with support.SuppressCrashReport():
        ok, stdout, stderr = assert_python_failure(*args)
    if b'ValueError: the number of frames must be in range' in stderr:
        return
    if b'-X tracemalloc=NFRAME: invalid number of frames' in stderr:
        return

    raise AssertionError(f'unexpected output: {stderr!a}')
for xoptions, nframe in (('tracemalloc', 1), ('tracemalloc=1', 1), ('tracemalloc=15', 15)):
    code = 'import tracemalloc; print(tracemalloc.get_traceback_limit())'
    ok, stdout, stderr = assert_python_ok('-X', xoptions, '-c', code)
    stdout = stdout.rstrip()

    assert stdout == str(nframe).encode('ascii')
print("TestCommandLine::test_sys_xoptions: ok")
