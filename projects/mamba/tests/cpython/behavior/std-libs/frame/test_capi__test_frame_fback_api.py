# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "test_capi__test_frame_fback_api"
# subject = "cpython.test_frame.TestCAPI.test_frame_fback_api"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frame.py::TestCAPI::test_frame_fback_api
"""Auto-ported test: TestCAPI::test_frame_fback_api (CPython 3.12 oracle)."""


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
"""Test that accessing `f_back` does not cause a segmentation fault on
        a frame created with `PyFrame_New` (GH-99110)."""

def dummy():
    pass
frame = _testcapi.frame_new(dummy.__code__, globals(), locals())

assert frame.f_back is None
print("TestCAPI::test_frame_fback_api: ok")
