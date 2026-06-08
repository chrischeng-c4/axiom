# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "clear_test__test_lineno_with_tracing"
# subject = "cpython.test_frame.ClearTest.test_lineno_with_tracing"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frame.py::ClearTest::test_lineno_with_tracing
"""Auto-ported test: ClearTest::test_lineno_with_tracing (CPython 3.12 oracle)."""


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
def record_line():
    f = sys._getframe(1)
    lines.append(f.f_lineno - f.f_code.co_firstlineno)

def test(trace):
    record_line()
    if trace:
        sys._getframe(0).f_trace = True
    record_line()
    record_line()
expected_lines = [1, 4, 5]
lines = []
test(False)

assert lines == expected_lines
lines = []
test(True)

assert lines == expected_lines
print("ClearTest::test_lineno_with_tracing: ok")
