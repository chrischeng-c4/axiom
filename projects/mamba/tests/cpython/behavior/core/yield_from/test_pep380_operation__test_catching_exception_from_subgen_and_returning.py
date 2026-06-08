# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_catching_exception_from_subgen_and_returning"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_catching_exception_from_subgen_and_returning"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_catching_exception_from_subgen_and_returning
"""Auto-ported test: TestPEP380Operation::test_catching_exception_from_subgen_and_returning (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test catching an exception thrown into a
        subgenerator and returning a value
        """

def inner():
    try:
        yield 1
    except ValueError:
        trace.append('inner caught ValueError')
    return value

def outer():
    v = (yield from inner())
    trace.append('inner returned %r to outer' % (v,))
    yield v
for value in (2, (2,), StopIteration(2)):
    trace = []
    g = outer()
    trace.append(next(g))
    trace.append(repr(g.throw(ValueError)))

    assert trace == [1, 'inner caught ValueError', 'inner returned %r to outer' % (value,), repr(value)]
print("TestPEP380Operation::test_catching_exception_from_subgen_and_returning: ok")
