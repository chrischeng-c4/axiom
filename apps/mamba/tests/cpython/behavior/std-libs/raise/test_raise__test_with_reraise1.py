# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_raise__test_with_reraise1"
# subject = "cpython.test_raise.TestRaise.test_with_reraise1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_raise.py::TestRaise::test_with_reraise1
"""Auto-ported test: TestRaise::test_with_reraise1 (CPython 3.12 oracle)."""


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
        with Context():
            pass
        raise

try:
    reraise()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestRaise::test_with_reraise1: ok")
