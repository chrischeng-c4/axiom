# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_context__test_noraise_finally"
# subject = "cpython.test_raise.TestContext.test_noraise_finally"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_raise.py::TestContext::test_noraise_finally
"""Auto-ported test: TestContext::test_noraise_finally (CPython 3.12 oracle)."""


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
        pass
    finally:
        raise OSError
except OSError as e:

    assert e.__context__ is None
else:

    raise AssertionError('No exception raised')
print("TestContext::test_noraise_finally: ok")
