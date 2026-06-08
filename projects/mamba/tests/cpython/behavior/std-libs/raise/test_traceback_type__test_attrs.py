# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_traceback_type__test_attrs"
# subject = "cpython.test_raise.TestTracebackType.test_attrs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_raise.py::TestTracebackType::test_attrs
"""Auto-ported test: TestTracebackType::test_attrs (CPython 3.12 oracle)."""


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
def raiser():
    raise ValueError
try:
    raiser()
except Exception as exc:
    tb = exc.__traceback__

assert isinstance(tb.tb_next, types.TracebackType)

assert tb.tb_frame is sys._getframe()

assert isinstance(tb.tb_lasti, int)

assert isinstance(tb.tb_lineno, int)

assert tb.tb_next.tb_next is None
try:
    del tb.tb_next
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    tb.tb_next = 'asdf'
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    tb.tb_next = tb
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    tb.tb_next.tb_next = tb
    raise AssertionError('expected ValueError')
except ValueError:
    pass
tb.tb_next = None

assert tb.tb_next is None
new_tb = get_tb()
tb.tb_next = new_tb

assert tb.tb_next is new_tb
print("TestTracebackType::test_attrs: ok")
