# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_cause__test_cause_syntax"
# subject = "cpython.test_raise.TestCause.testCauseSyntax"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_raise.py::TestCause::testCauseSyntax
"""Auto-ported test: TestCause::testCauseSyntax (CPython 3.12 oracle)."""


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
        try:
            raise TypeError
        except Exception:
            raise ValueError from None
    except ValueError as exc:

        assert exc.__cause__ is None

        assert exc.__suppress_context__
        exc.__suppress_context__ = False
        raise exc
except ValueError as exc:
    e = exc

assert e.__cause__ is None

assert not e.__suppress_context__

assert isinstance(e.__context__, TypeError)
print("TestCause::testCauseSyntax: ok")
