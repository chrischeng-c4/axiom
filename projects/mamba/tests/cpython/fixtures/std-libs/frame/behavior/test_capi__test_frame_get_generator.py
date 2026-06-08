# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "test_capi__test_frame_get_generator"
# subject = "cpython.test_frame.TestCAPI.test_frame_get_generator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frame.py::TestCAPI::test_frame_get_generator
"""Auto-ported test: TestCAPI::test_frame_get_generator (CPython 3.12 oracle)."""


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
gen = getgenframe()
frame = next(gen)

assert gen is _testcapi.frame_getgenerator(frame)
print("TestCAPI::test_frame_get_generator: ok")
