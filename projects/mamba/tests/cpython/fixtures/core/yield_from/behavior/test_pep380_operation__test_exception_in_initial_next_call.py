# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_exception_in_initial_next_call"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_exception_in_initial_next_call"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_exception_in_initial_next_call
"""Auto-ported test: TestPEP380Operation::test_exception_in_initial_next_call (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test exception in initial next() call
        """
trace = []

def g1():
    trace.append('g1 about to yield from g2')
    yield from g2()
    trace.append('g1 should not be here')

def g2():
    yield (1 / 0)

def run():
    gi = g1()
    next(gi)

try:
    run()
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert trace == ['g1 about to yield from g2']
print("TestPEP380Operation::test_exception_in_initial_next_call: ok")
