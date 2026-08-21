# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_returning_value_from_delegated_throw"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_returning_value_from_delegated_throw"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_returning_value_from_delegated_throw
"""Auto-ported test: TestPEP380Operation::test_returning_value_from_delegated_throw (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test returning value from delegated 'throw'
        """
trace = []

def g1():
    try:
        trace.append('Starting g1')
        yield 'g1 ham'
        yield from g2()
        yield 'g1 eggs'
    finally:
        trace.append('Finishing g1')

def g2():
    try:
        trace.append('Starting g2')
        yield 'g2 spam'
        yield 'g2 more spam'
    except LunchError:
        trace.append('Caught LunchError in g2')
        yield 'g2 lunch saved'
        yield 'g2 yet more spam'

class LunchError(Exception):
    pass
g = g1()
for i in range(2):
    x = next(g)
    trace.append('Yielded %s' % (x,))
e = LunchError('tomato ejected')
g.throw(e)
for x in g:
    trace.append('Yielded %s' % (x,))

assert trace == ['Starting g1', 'Yielded g1 ham', 'Starting g2', 'Yielded g2 spam', 'Caught LunchError in g2', 'Yielded g2 yet more spam', 'Yielded g1 eggs', 'Finishing g1']
print("TestPEP380Operation::test_returning_value_from_delegated_throw: ok")
