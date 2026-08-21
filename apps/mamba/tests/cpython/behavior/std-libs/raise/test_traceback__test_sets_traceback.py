# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_traceback__test_sets_traceback"
# subject = "cpython.test_raise.TestTraceback.test_sets_traceback"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_raise.py::TestTraceback::test_sets_traceback
"""Auto-ported test: TestTraceback::test_sets_traceback (CPython 3.12 oracle)."""


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
try:
    raise IndexError()
except IndexError as e:

    assert isinstance(e.__traceback__, types.TracebackType)
else:

    raise AssertionError('No exception raised')
print("TestTraceback::test_sets_traceback: ok")
