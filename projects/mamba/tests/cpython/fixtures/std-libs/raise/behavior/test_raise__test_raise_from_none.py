# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_raise__test_raise_from_none"
# subject = "cpython.test_raise.TestRaise.test_raise_from_None"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_raise.py::TestRaise::test_raise_from_None
"""Auto-ported test: TestRaise::test_raise_from_None (CPython 3.12 oracle)."""


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
    try:
        raise TypeError('foo')
    except:
        raise ValueError() from None
except ValueError as e:

    assert isinstance(e.__context__, TypeError)

    assert e.__cause__ is None
print("TestRaise::test_raise_from_None: ok")
