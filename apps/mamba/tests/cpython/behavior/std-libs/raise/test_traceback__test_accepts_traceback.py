# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_traceback__test_accepts_traceback"
# subject = "cpython.test_raise.TestTraceback.test_accepts_traceback"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_raise.py::TestTraceback::test_accepts_traceback
"""Auto-ported test: TestTraceback::test_accepts_traceback (CPython 3.12 oracle)."""


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
tb = get_tb()
try:
    raise IndexError().with_traceback(tb)
except IndexError as e:

    assert e.__traceback__ != tb

    assert e.__traceback__.tb_next == tb
else:

    raise AssertionError('No exception raised')
print("TestTraceback::test_accepts_traceback: ok")
