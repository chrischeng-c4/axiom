# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_handing_exception_while_delegating_close"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_handing_exception_while_delegating_close"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_handing_exception_while_delegating_close
"""Auto-ported test: TestPEP380Operation::test_handing_exception_while_delegating_close (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test handling exception while delegating 'close'
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
    finally:
        trace.append('Finishing g2')
        raise ValueError('nybbles have exploded with delight')
try:
    g = g1()
    for i in range(2):
        x = next(g)
        trace.append('Yielded %s' % (x,))
    g.close()
except ValueError as e:

    assert e.args[0] == 'nybbles have exploded with delight'

    assert isinstance(e.__context__, GeneratorExit)
else:

    raise AssertionError('subgenerator failed to raise ValueError')

assert trace == ['Starting g1', 'Yielded g1 ham', 'Starting g2', 'Yielded g2 spam', 'Finishing g2', 'Finishing g1']
print("TestPEP380Operation::test_handing_exception_while_delegating_close: ok")
