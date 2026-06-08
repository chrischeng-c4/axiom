# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_exception_locations"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_exception_locations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_exception_locations
"""Auto-ported test: DictComprehensionTest::test_exception_locations."""


import traceback
from test.support import BrokenIter


def init_raises():
    try:
        {x: x for x in BrokenIter(init_raises=True)}
    except Exception as exc:
        return exc


def next_raises():
    try:
        {x: x for x in BrokenIter(next_raises=True)}
    except Exception as exc:
        return exc


def iter_raises():
    try:
        {x: x for x in BrokenIter(iter_raises=True)}
    except Exception as exc:
        return exc


for func, expected in [
    (init_raises, "BrokenIter(init_raises=True)"),
    (next_raises, "BrokenIter(next_raises=True)"),
    (iter_raises, "BrokenIter(iter_raises=True)"),
]:
    exc = func()
    frame = traceback.extract_tb(exc.__traceback__)[0]
    indent = 8
    code = func.__code__
    assert frame.lineno == code.co_firstlineno + 2
    assert frame.end_lineno == code.co_firstlineno + 2
    assert frame.line[frame.colno - indent : frame.end_colno - indent] == expected

print("DictComprehensionTest::test_exception_locations: ok")
