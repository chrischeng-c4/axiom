# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "setcomps"
# dimension = "behavior"
# case = "set_comprehension_test__test_exception_locations"
# subject = "cpython.test_setcomps.SetComprehensionTest.test_exception_locations"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_setcomps.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_setcomps.py::SetComprehensionTest::test_exception_locations
"""Auto-ported test: SetComprehensionTest::test_exception_locations."""


import traceback


class BrokenIter:
    def __init__(self, init_raises=False, iter_raises=False, next_raises=False):
        if init_raises:
            raise Exception("init")
        self.iter_raises = iter_raises
        self.next_raises = next_raises

    def __iter__(self):
        if self.iter_raises:
            raise Exception("iter")
        return self

    def __next__(self):
        if self.next_raises:
            raise Exception("next")
        raise StopIteration


def init_raises():
    try:
        {x for x in BrokenIter(init_raises=True)}
    except Exception as exc:
        return exc


def next_raises():
    try:
        {x for x in BrokenIter(next_raises=True)}
    except Exception as exc:
        return exc


def iter_raises():
    try:
        {x for x in BrokenIter(iter_raises=True)}
    except Exception as exc:
        return exc


for func, expected in [
    (init_raises, "BrokenIter(init_raises=True)"),
    (next_raises, "BrokenIter(next_raises=True)"),
    (iter_raises, "BrokenIter(iter_raises=True)"),
]:
    exc = func()
    frame = traceback.extract_tb(exc.__traceback__)[0]
    assert frame.lineno == func.__code__.co_firstlineno + 2
    assert frame.end_lineno == func.__code__.co_firstlineno + 2
    assert expected in frame.line

print("SetComprehensionTest::test_exception_locations: ok")
