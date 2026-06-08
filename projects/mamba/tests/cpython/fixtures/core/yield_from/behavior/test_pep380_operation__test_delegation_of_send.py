# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegation_of_send"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegation_of_send"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegation_of_send
"""Auto-ported test: TestPEP380Operation::test_delegation_of_send (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegation of send()
        """
trace = []

def g1():
    trace.append('Starting g1')
    x = (yield 'g1 ham')
    trace.append('g1 received %s' % (x,))
    yield from g2()
    x = (yield 'g1 eggs')
    trace.append('g1 received %s' % (x,))
    trace.append('Finishing g1')

def g2():
    trace.append('Starting g2')
    x = (yield 'g2 spam')
    trace.append('g2 received %s' % (x,))
    x = (yield 'g2 more spam')
    trace.append('g2 received %s' % (x,))
    trace.append('Finishing g2')
g = g1()
y = next(g)
x = 1
try:
    while 1:
        y = g.send(x)
        trace.append('Yielded %s' % (y,))
        x += 1
except StopIteration:
    pass

assert trace == ['Starting g1', 'g1 received 1', 'Starting g2', 'Yielded g2 spam', 'g2 received 2', 'Yielded g2 more spam', 'g2 received 3', 'Finishing g2', 'Yielded g1 eggs', 'g1 received 4', 'Finishing g1']
print("TestPEP380Operation::test_delegation_of_send: ok")
