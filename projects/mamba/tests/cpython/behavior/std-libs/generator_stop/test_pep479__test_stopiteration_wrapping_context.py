# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "generator_stop"
# dimension = "behavior"
# case = "test_pep479__test_stopiteration_wrapping_context"
# subject = "cpython.test_generator_stop.TestPEP479.test_stopiteration_wrapping_context"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_generator_stop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_generator_stop.py::TestPEP479::test_stopiteration_wrapping_context
"""Auto-ported test: TestPEP479::test_stopiteration_wrapping_context (CPython 3.12 oracle)."""


from __future__ import generator_stop
import unittest


# --- test body ---
def f():
    raise StopIteration

def g():
    yield f()
try:
    next(g())
except RuntimeError as exc:

    assert type(exc.__cause__) is StopIteration

    assert type(exc.__context__) is StopIteration

    assert exc.__suppress_context__
else:

    raise AssertionError('__cause__, __context__, or __suppress_context__ were not properly set')
print("TestPEP479::test_stopiteration_wrapping_context: ok")
