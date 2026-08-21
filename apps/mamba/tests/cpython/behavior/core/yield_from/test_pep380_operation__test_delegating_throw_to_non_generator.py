# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegating_throw_to_non_generator"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegating_throw_to_non_generator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegating_throw_to_non_generator
"""Auto-ported test: TestPEP380Operation::test_delegating_throw_to_non_generator (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegating 'throw' to non-generator
        """
trace = []

def g():
    try:
        trace.append('Starting g')
        yield from range(10)
    finally:
        trace.append('Finishing g')
try:
    gi = g()
    for i in range(5):
        x = next(gi)
        trace.append('Yielded %s' % (x,))
    e = ValueError('tomato ejected')
    gi.throw(e)
except ValueError as e:

    assert e.args[0] == 'tomato ejected'
else:

    raise AssertionError('subgenerator failed to raise ValueError')

assert trace == ['Starting g', 'Yielded 0', 'Yielded 1', 'Yielded 2', 'Yielded 3', 'Yielded 4', 'Finishing g']
print("TestPEP380Operation::test_delegating_throw_to_non_generator: ok")
