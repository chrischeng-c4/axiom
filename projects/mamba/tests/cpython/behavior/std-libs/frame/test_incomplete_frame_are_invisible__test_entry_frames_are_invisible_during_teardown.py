# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "test_incomplete_frame_are_invisible__test_entry_frames_are_invisible_during_teardown"
# subject = "cpython.test_frame.TestIncompleteFrameAreInvisible.test_entry_frames_are_invisible_during_teardown"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frame.py::TestIncompleteFrameAreInvisible::test_entry_frames_are_invisible_during_teardown
"""Auto-ported test: TestIncompleteFrameAreInvisible::test_entry_frames_are_invisible_during_teardown (CPython 3.12 oracle)."""


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
class C:
    """A weakref'able class."""

def f():
    """Try to find globals and locals as this frame is being cleared."""
    ref = C()
    return weakref.ref(ref, exec)
with support.catch_unraisable_exception() as catcher:
    weak = operator.call(f)

    assert catcher.unraisable.exc_type is TypeError

assert weak() is None
print("TestIncompleteFrameAreInvisible::test_entry_frames_are_invisible_during_teardown: ok")
