# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_raise__test_yield_reraise"
# subject = "cpython.test_raise.TestRaise.test_yield_reraise"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_raise.py::TestRaise::test_yield_reraise
"""Auto-ported test: TestRaise::test_yield_reraise (CPython 3.12 oracle)."""


from test import support
import sys
import types
import unittest


'Tests for the raise statement.'

def get_tb():
    try:
        raise OSError()
    except OSError as e:
        return e.__traceback__

class Context:

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        return True


# --- test body ---
def reraise():
    try:
        raise TypeError('foo')
    except:
        yield 1
        raise
g = reraise()
next(g)

try:
    (lambda: next(g))()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: next(g))()
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
print("TestRaise::test_yield_reraise: ok")
