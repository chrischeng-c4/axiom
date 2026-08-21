# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_attempted_yield_from_loop"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_attempted_yield_from_loop"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_attempted_yield_from_loop
"""Auto-ported test: TestPEP380Operation::test_attempted_yield_from_loop (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test attempted yield-from loop
        """
trace = []

def g1():
    trace.append('g1: starting')
    yield 'y1'
    trace.append('g1: about to yield from g2')
    yield from g2()
    trace.append('g1 should not be here')

def g2():
    trace.append('g2: starting')
    yield 'y2'
    trace.append('g2: about to yield from g1')
    yield from gi
    trace.append('g2 should not be here')
try:
    gi = g1()
    for y in gi:
        trace.append('Yielded: %s' % (y,))
except ValueError as e:

    assert e.args[0] == 'generator already executing'
else:

    raise AssertionError("subgenerator didn't raise ValueError")

assert trace == ['g1: starting', 'Yielded: y1', 'g1: about to yield from g2', 'g2: starting', 'Yielded: y2', 'g2: about to yield from g1']
print("TestPEP380Operation::test_attempted_yield_from_loop: ok")
