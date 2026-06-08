# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "generator_stop"
# dimension = "behavior"
# case = "test_pep479__test_stopiteration_wrapping"
# subject = "cpython.test_generator_stop.TestPEP479.test_stopiteration_wrapping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_generator_stop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_generator_stop.py::TestPEP479::test_stopiteration_wrapping
"""Auto-ported test: TestPEP479::test_stopiteration_wrapping (CPython 3.12 oracle)."""


from __future__ import generator_stop
import unittest


# --- test body ---
def f():
    raise StopIteration

def g():
    yield f()
try:
    next(g())
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('generator raised StopIteration', str(_aR_e))
print("TestPEP479::test_stopiteration_wrapping: ok")
