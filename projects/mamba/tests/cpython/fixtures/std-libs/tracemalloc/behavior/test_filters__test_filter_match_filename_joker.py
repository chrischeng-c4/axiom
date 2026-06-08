# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "test_filters__test_filter_match_filename_joker"
# subject = "cpython.test_tracemalloc.TestFilters.test_filter_match_filename_joker"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tracemalloc.py::TestFilters::test_filter_match_filename_joker
"""Auto-ported test: TestFilters::test_filter_match_filename_joker (CPython 3.12 oracle)."""


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

def fnmatch(filename, pattern):
    filter = tracemalloc.Filter(True, pattern)
    return filter._match_frame(filename, 0)

assert not fnmatch('abc', '')

assert not fnmatch('', 'abc')

assert fnmatch('', '')

assert fnmatch('', '*')

assert fnmatch('abc', 'abc')

assert not fnmatch('abc', 'abcd')

assert not fnmatch('abc', 'def')

assert fnmatch('abc', 'a*')

assert fnmatch('abc', 'abc*')

assert not fnmatch('abc', 'b*')

assert not fnmatch('abc', 'abcd*')

assert fnmatch('abc', 'a*c')

assert fnmatch('abcdcx', 'a*cx')

assert not fnmatch('abb', 'a*c')

assert not fnmatch('abcdce', 'a*cx')

assert fnmatch('abcde', 'a*c*e')

assert fnmatch('abcbdefeg', 'a*bd*eg')

assert not fnmatch('abcdd', 'a*c*e')

assert not fnmatch('abcbdefef', 'a*bd*eg')

assert fnmatch('a.pyc', 'a.py')

assert fnmatch('a.py', 'a.pyc')
if os.name == 'nt':

    assert fnmatch('aBC', 'ABc')

    assert fnmatch('aBcDe', 'Ab*dE')

    assert fnmatch('a.pyc', 'a.PY')

    assert fnmatch('a.py', 'a.PYC')
else:

    assert not fnmatch('aBC', 'ABc')

    assert not fnmatch('aBcDe', 'Ab*dE')

    assert not fnmatch('a.pyc', 'a.PY')

    assert not fnmatch('a.py', 'a.PYC')
if os.name == 'nt':

    assert fnmatch('a/b', 'a\\b')

    assert fnmatch('a\\b', 'a/b')

    assert fnmatch('a/b\\c', 'a\\b/c')

    assert fnmatch('a/b/c', 'a\\b\\c')
else:

    assert not fnmatch('a/b', 'a\\b')

    assert not fnmatch('a\\b', 'a/b')

    assert not fnmatch('a/b\\c', 'a\\b/c')

    assert not fnmatch('a/b/c', 'a\\b\\c')

assert not fnmatch('a.pyo', 'a.py')
print("TestFilters::test_filter_match_filename_joker: ok")
