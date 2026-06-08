# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegation_of_next_to_non_generator"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegation_of_next_to_non_generator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegation_of_next_to_non_generator
"""Auto-ported test: TestPEP380Operation::test_delegation_of_next_to_non_generator (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegation of next() to non-generator
        """
trace = []

def g():
    yield from range(3)
for x in g():
    trace.append('Yielded %s' % (x,))

assert trace == ['Yielded 0', 'Yielded 1', 'Yielded 2']
print("TestPEP380Operation::test_delegation_of_next_to_non_generator: ok")
