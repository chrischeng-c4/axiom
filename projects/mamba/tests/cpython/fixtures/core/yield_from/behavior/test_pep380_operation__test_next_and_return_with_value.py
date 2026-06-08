# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_next_and_return_with_value"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_next_and_return_with_value"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_next_and_return_with_value
"""Auto-ported test: TestPEP380Operation::test_next_and_return_with_value (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test next and return with value
        """
trace = []

def f(r):
    gi = g(r)
    next(gi)
    try:
        trace.append('f resuming g')
        next(gi)
        trace.append('f SHOULD NOT BE HERE')
    except StopIteration as e:
        trace.append('f caught %r' % (e,))

def g(r):
    trace.append('g starting')
    yield
    trace.append('g returning %r' % (r,))
    return r
f(None)
f(1)
f((2,))
f(StopIteration(3))

assert trace == ['g starting', 'f resuming g', 'g returning None', 'f caught StopIteration()', 'g starting', 'f resuming g', 'g returning 1', 'f caught StopIteration(1)', 'g starting', 'f resuming g', 'g returning (2,)', 'f caught StopIteration((2,))', 'g starting', 'f resuming g', 'g returning StopIteration(3)', 'f caught StopIteration(StopIteration(3))']
print("TestPEP380Operation::test_next_and_return_with_value: ok")
