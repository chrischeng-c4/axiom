# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "test_capi__test_frame_getters"
# subject = "cpython.test_frame.TestCAPI.test_frame_getters"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frame.py::TestCAPI::test_frame_getters
"""Auto-ported test: TestCAPI::test_frame_getters (CPython 3.12 oracle)."""


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
def getframe():
    return sys._getframe()

def getgenframe():
    yield sys._getframe()
frame = getframe()

assert frame.f_locals == _testcapi.frame_getlocals(frame)

assert frame.f_globals is _testcapi.frame_getglobals(frame)

assert frame.f_builtins is _testcapi.frame_getbuiltins(frame)

assert frame.f_lasti == _testcapi.frame_getlasti(frame)
print("TestCAPI::test_frame_getters: ok")
