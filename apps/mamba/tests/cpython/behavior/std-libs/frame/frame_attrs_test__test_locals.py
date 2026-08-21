# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "frame_attrs_test__test_locals"
# subject = "cpython.test_frame.FrameAttrsTest.test_locals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frame.py::FrameAttrsTest::test_locals
"""Auto-ported test: FrameAttrsTest::test_locals (CPython 3.12 oracle)."""


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
def make_frames():

    def outer():
        x = 5
        y = 6

        def inner():
            z = x + 2
            1 / 0
            t = 9
        return inner()
    try:
        outer()
    except ZeroDivisionError as e:
        tb = e.__traceback__
        frames = []
        while tb:
            frames.append(tb.tb_frame)
            tb = tb.tb_next
    return frames
f, outer, inner = make_frames()
outer_locals = outer.f_locals

assert isinstance(outer_locals.pop('inner'), types.FunctionType)

assert outer_locals == {'x': 5, 'y': 6}
inner_locals = inner.f_locals

assert inner_locals == {'x': 5, 'z': 7}
print("FrameAttrsTest::test_locals: ok")
