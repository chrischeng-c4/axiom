# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "clear_test__test_clear_locals"
# subject = "cpython.test_frame.ClearTest.test_clear_locals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frame.py::ClearTest::test_clear_locals
"""Auto-ported test: ClearTest::test_clear_locals (CPython 3.12 oracle)."""


import gc
import operator
import re
import sys
import textwrap
import threading
import types
import unittest
import weakref
from test import support
from test.support import threading_helper
from test.support.script_helper import assert_python_ok


try:
    import _testcapi
except ImportError:
    _testcapi = None


# --- test body ---
def clear_traceback_frames(tb):
    """
        Clear all frames in a traceback.
        """
    while tb is not None:
        tb.tb_frame.clear()
        tb = tb.tb_next

def inner(x=5, **kwargs):
    1 / 0

def outer(**kwargs):
    try:
        inner(**kwargs)
    except ZeroDivisionError as e:
        exc = e
    return exc

class C:
    pass
c = C()
wr = weakref.ref(c)
exc = outer(c=c)
del c
support.gc_collect()

assert None is not wr()
clear_traceback_frames(exc.__traceback__)
support.gc_collect()

assert None is wr()
print("ClearTest::test_clear_locals: ok")
