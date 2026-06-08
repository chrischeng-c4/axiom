# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "test_snapshot__test_snapshot_group_by_traceback"
# subject = "cpython.test_tracemalloc.TestSnapshot.test_snapshot_group_by_traceback"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tracemalloc.py::TestSnapshot::test_snapshot_group_by_traceback
"""Auto-ported test: TestSnapshot::test_snapshot_group_by_traceback (CPython 3.12 oracle)."""


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
maxDiff = 4000
snapshot, snapshot2 = create_snapshots()
tb1 = traceback(('a.py', 2), ('b.py', 4))
tb2 = traceback(('a.py', 5), ('b.py', 4))
tb3 = traceback(('b.py', 1))
tb4 = traceback(('<unknown>', 0))
stats1 = snapshot.statistics('traceback')

assert stats1 == [tracemalloc.Statistic(tb3, 66, 1), tracemalloc.Statistic(tb1, 30, 3), tracemalloc.Statistic(tb4, 7, 1), tracemalloc.Statistic(tb2, 2, 1)]
tb5 = traceback(('c.py', 578))
stats2 = snapshot2.statistics('traceback')

assert stats2 == [tracemalloc.Statistic(tb2, 5002, 2), tracemalloc.Statistic(tb5, 400, 1), tracemalloc.Statistic(tb1, 30, 3)]
diff = snapshot2.compare_to(snapshot, 'traceback')

assert diff == [tracemalloc.StatisticDiff(tb2, 5002, 5000, 2, 1), tracemalloc.StatisticDiff(tb5, 400, 400, 1, 1), tracemalloc.StatisticDiff(tb3, 0, -66, 0, -1), tracemalloc.StatisticDiff(tb4, 0, -7, 0, -1), tracemalloc.StatisticDiff(tb1, 30, 0, 3, 0)]

try:
    snapshot.statistics('traceback', cumulative=True)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestSnapshot::test_snapshot_group_by_traceback: ok")
