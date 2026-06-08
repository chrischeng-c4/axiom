# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_context__test_not_last"
# subject = "cpython.test_raise.TestContext.test_not_last"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_raise.py::TestContext::test_not_last
"""Auto-ported test: TestContext::test_not_last (CPython 3.12 oracle)."""


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
context = Exception('context')
try:
    raise context
except Exception:
    try:
        raise Exception('caught')
    except Exception:
        pass
    try:
        raise Exception('new')
    except Exception as exc:
        raised = exc

assert raised.__context__ is context
print("TestContext::test_not_last: ok")
