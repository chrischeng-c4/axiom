# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_broken_getattr_handling"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_broken_getattr_handling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_broken_getattr_handling
"""Auto-ported test: TestPEP380Operation::test_broken_getattr_handling (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test subiterator with a broken getattr implementation
        """

class Broken:

    def __iter__(self):
        return self

    def __next__(self):
        return 1

    def __getattr__(self, attr):
        1 / 0

def g():
    yield from Broken()
try:
    gi = g()

    assert next(gi) == 1
    gi.send(1)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass
try:
    gi = g()

    assert next(gi) == 1
    gi.throw(AttributeError)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass
with support.catch_unraisable_exception() as cm:
    gi = g()

    assert next(gi) == 1
    gi.close()

    assert ZeroDivisionError == cm.unraisable.exc_type
print("TestPEP380Operation::test_broken_getattr_handling: ok")
