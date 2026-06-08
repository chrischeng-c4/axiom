# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_close_with_cleared_frame"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_close_with_cleared_frame"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_close_with_cleared_frame
"""Auto-ported test: TestPEP380Operation::test_close_with_cleared_frame (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
def innermost():
    yield

def inner():
    outer_gen = (yield)
    yield from innermost()

def outer():
    inner_gen = (yield)
    yield from inner_gen
with disable_gc():
    inner_gen = inner()
    outer_gen = outer()
    outer_gen.send(None)
    outer_gen.send(inner_gen)
    outer_gen.send(outer_gen)
    del outer_gen
    del inner_gen
    gc_collect()
print("TestPEP380Operation::test_close_with_cleared_frame: ok")
