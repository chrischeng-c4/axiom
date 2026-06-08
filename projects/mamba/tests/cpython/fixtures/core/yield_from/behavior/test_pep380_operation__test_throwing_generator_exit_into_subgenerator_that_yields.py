# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_throwing_generator_exit_into_subgenerator_that_yields"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_throwing_GeneratorExit_into_subgenerator_that_yields"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_throwing_GeneratorExit_into_subgenerator_that_yields
"""Auto-ported test: TestPEP380Operation::test_throwing_GeneratorExit_into_subgenerator_that_yields (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test throwing GeneratorExit into a subgenerator that
        catches it and yields.
        """
trace = []

def f():
    try:
        trace.append('Enter f')
        yield
        trace.append('Exit f')
    except GeneratorExit:
        yield

def g():
    trace.append('Enter g')
    yield from f()
    trace.append('Exit g')
try:
    gi = g()
    next(gi)
    gi.throw(GeneratorExit)
except RuntimeError as e:

    assert e.args[0] == 'generator ignored GeneratorExit'
else:

    raise AssertionError('subgenerator failed to raise GeneratorExit')

assert trace == ['Enter g', 'Enter f']
print("TestPEP380Operation::test_throwing_GeneratorExit_into_subgenerator_that_yields: ok")
