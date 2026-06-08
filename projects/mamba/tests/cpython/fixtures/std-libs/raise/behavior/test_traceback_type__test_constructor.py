# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_traceback_type__test_constructor"
# subject = "cpython.test_raise.TestTracebackType.test_constructor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_raise.py::TestTracebackType::test_constructor
"""Auto-ported test: TestTracebackType::test_constructor (CPython 3.12 oracle)."""


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
other_tb = get_tb()
frame = sys._getframe()
tb = types.TracebackType(other_tb, frame, 1, 2)

assert tb.tb_next == other_tb

assert tb.tb_frame == frame

assert tb.tb_lasti == 1

assert tb.tb_lineno == 2
tb = types.TracebackType(None, frame, 1, 2)

assert tb.tb_next == None
try:
    types.TracebackType('no', frame, 1, 2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    types.TracebackType(other_tb, 'no', 1, 2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    types.TracebackType(other_tb, frame, 'no', 2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    types.TracebackType(other_tb, frame, 1, 'nuh-uh')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestTracebackType::test_constructor: ok")
