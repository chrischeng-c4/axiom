# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "iter"
# dimension = "behavior"
# case = "test_case__test_exception_locations"
# subject = "cpython.test_iter.TestCase.test_exception_locations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestCase::test_exception_locations (CPython 3.12 oracle)."""

import traceback


class BrokenIter:
    def __init__(self, init_raises=False, next_raises=False, iter_raises=False):
        if init_raises:
            1 / 0
        self.next_raises = next_raises
        self.iter_raises = iter_raises

    def __next__(self):
        if self.next_raises:
            1 / 0

    def __iter__(self):
        if self.iter_raises:
            1 / 0
        return self


def init_raises():
    try:
        for x in BrokenIter(init_raises=True):
            pass
    except Exception as exc:
        return exc


def next_raises():
    try:
        for x in BrokenIter(next_raises=True):
            pass
    except Exception as exc:
        return exc


def iter_raises():
    try:
        for x in BrokenIter(iter_raises=True):
            pass
    except Exception as exc:
        return exc


for func, expected in [
    (init_raises, "BrokenIter(init_raises=True)"),
    (next_raises, "BrokenIter(next_raises=True)"),
    (iter_raises, "BrokenIter(iter_raises=True)"),
]:
    exc = func()
    assert exc is not None, func.__name__
    frame = traceback.extract_tb(exc.__traceback__)[0]
    assert frame.lineno == func.__code__.co_firstlineno + 2, frame
    assert frame.end_lineno == func.__code__.co_firstlineno + 2, frame
    assert frame.line[frame.colno - 8:frame.end_colno - 8] == expected, frame

print("TestCase::test_exception_locations: ok")
