# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "clear_test__test_clear_does_not_clear_specials"
# subject = "cpython.test_frame.ClearTest.test_clear_does_not_clear_specials"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frame.py::ClearTest::test_clear_does_not_clear_specials
"""Auto-ported test: ClearTest::test_clear_does_not_clear_specials (CPython 3.12 oracle)."""


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
exc = outer(c=c)
del c
f = exc.__traceback__.tb_frame
f.clear()

assert f.f_code is not None

assert f.f_locals is not None

assert f.f_builtins is not None

assert f.f_globals is not None
print("ClearTest::test_clear_does_not_clear_specials: ok")
