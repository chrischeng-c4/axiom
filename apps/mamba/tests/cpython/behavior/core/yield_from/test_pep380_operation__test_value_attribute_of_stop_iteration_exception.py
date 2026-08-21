# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_value_attribute_of_stop_iteration_exception"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_value_attribute_of_StopIteration_exception"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_value_attribute_of_StopIteration_exception
"""Auto-ported test: TestPEP380Operation::test_value_attribute_of_StopIteration_exception (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test 'value' attribute of StopIteration exception
        """
trace = []

def pex(e):
    trace.append('%s: %s' % (e.__class__.__name__, e))
    trace.append('value = %s' % (e.value,))
e = StopIteration()
pex(e)
e = StopIteration('spam')
pex(e)
e.value = 'eggs'
pex(e)

assert trace == ['StopIteration: ', 'value = None', 'StopIteration: spam', 'value = spam', 'StopIteration: spam', 'value = eggs']
print("TestPEP380Operation::test_value_attribute_of_StopIteration_exception: ok")
