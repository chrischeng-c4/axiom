# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "test_filters__test_filter_match"
# subject = "cpython.test_tracemalloc.TestFilters.test_filter_match"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tracemalloc.py::TestFilters::test_filter_match
"""Auto-ported test: TestFilters::test_filter_match (CPython 3.12 oracle)."""


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
maxDiff = 2048
f = tracemalloc.Filter(True, 'abc')

assert f._match_frame('abc', 0)

assert f._match_frame('abc', 5)

assert f._match_frame('abc', 10)

assert not f._match_frame('12356', 0)

assert not f._match_frame('12356', 5)

assert not f._match_frame('12356', 10)
f = tracemalloc.Filter(False, 'abc')

assert not f._match_frame('abc', 0)

assert not f._match_frame('abc', 5)

assert not f._match_frame('abc', 10)

assert f._match_frame('12356', 0)

assert f._match_frame('12356', 5)

assert f._match_frame('12356', 10)
f = tracemalloc.Filter(True, 'abc', 5)

assert not f._match_frame('abc', 0)

assert f._match_frame('abc', 5)

assert not f._match_frame('abc', 10)

assert not f._match_frame('12356', 0)

assert not f._match_frame('12356', 5)

assert not f._match_frame('12356', 10)
f = tracemalloc.Filter(False, 'abc', 5)

assert f._match_frame('abc', 0)

assert not f._match_frame('abc', 5)

assert f._match_frame('abc', 10)

assert f._match_frame('12356', 0)

assert f._match_frame('12356', 5)

assert f._match_frame('12356', 10)
f = tracemalloc.Filter(True, 'abc', 0)

assert f._match_frame('abc', 0)

assert not f._match_frame('abc', 5)

assert not f._match_frame('abc', 10)

assert not f._match_frame('12356', 0)

assert not f._match_frame('12356', 5)

assert not f._match_frame('12356', 10)
f = tracemalloc.Filter(False, 'abc', 0)

assert not f._match_frame('abc', 0)

assert f._match_frame('abc', 5)

assert f._match_frame('abc', 10)

assert f._match_frame('12356', 0)

assert f._match_frame('12356', 5)

assert f._match_frame('12356', 10)
print("TestFilters::test_filter_match: ok")
