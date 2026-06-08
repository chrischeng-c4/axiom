# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegation_of_next_call_to_subgenerator"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegation_of_next_call_to_subgenerator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegation_of_next_call_to_subgenerator
"""Auto-ported test: TestPEP380Operation::test_delegation_of_next_call_to_subgenerator (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegation of next() call to subgenerator
        """
trace = []

def g1():
    trace.append('Starting g1')
    yield 'g1 ham'
    yield from g2()
    yield 'g1 eggs'
    trace.append('Finishing g1')

def g2():
    trace.append('Starting g2')
    yield 'g2 spam'
    yield 'g2 more spam'
    trace.append('Finishing g2')
for x in g1():
    trace.append('Yielded %s' % (x,))

assert trace == ['Starting g1', 'Yielded g1 ham', 'Starting g2', 'Yielded g2 spam', 'Yielded g2 more spam', 'Finishing g2', 'Yielded g1 eggs', 'Finishing g1']
print("TestPEP380Operation::test_delegation_of_next_call_to_subgenerator: ok")
